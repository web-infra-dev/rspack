//! Native loader work is moved between the loader runner and JS isolates. No N-API
//! value is constructed until a worker takes a task's context.
use std::sync::{
  Arc, LazyLock, Mutex,
  atomic::{AtomicBool, AtomicU32, Ordering},
};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{LoaderContext, RunnerContext};
use rspack_js_tasks::{
  LoaderTask as NativeLoaderTask, LoaderTaskMode, TimingPoint, js_task_dispatcher,
};
use rustc_hash::FxHashMap;

use super::{
  JsLoaderContext,
  bridge::MainObjectHandle,
  result::{merge_loader_context, merge_loader_hooks},
};
use crate::module::WorkerModuleScope;

// Resources acquired in a JS environment stay in binding, outside the queue's
// native task protocol. Revoke the module scope before returning native state.
pub(crate) struct ActiveLoaderTask {
  native: Option<NativeLoaderTask>,
  _permit: Option<crate::js_tasks::TaskPermit>,
  module_scope: Option<Arc<WorkerModuleScope>>,
  bridge_handle: Option<MainObjectHandle>,
}

impl ActiveLoaderTask {
  fn native(&self) -> &NativeLoaderTask {
    self
      .native
      .as_ref()
      .expect("unfinished task owns native state")
  }

  fn context(&self) -> &LoaderContext<RunnerContext> {
    self.native().context()
  }

  fn context_mut(&mut self) -> &mut LoaderContext<RunnerContext> {
    self
      .native
      .as_mut()
      .expect("unfinished task owns native state")
      .context_mut()
  }

  fn revoke(&self) {
    if let Some(scope) = &self.module_scope {
      scope.revoke();
    }
  }

  fn finish(mut self, pitches: Vec<String>, result: rspack_error::Result<()>) {
    self.revoke();
    if let Some(timing) = &self.native().timing {
      timing.mark(TimingPoint::Returned);
    }
    self
      .native
      .take()
      .expect("unfinished task owns native state")
      .finish(pitches, result);
  }
}

impl Drop for ActiveLoaderTask {
  fn drop(&mut self) {
    self.revoke();
    // NativeLoaderTask::drop returns the context after this revoke.
  }
}

#[derive(Default)]
struct WorkerSlot {
  stopped: AtomicBool,
  tasks: Mutex<FxHashMap<u32, TaskCell>>,
}

static NEXT_TASK: AtomicU32 = AtomicU32::new(1);
pub(crate) type TaskCell = Arc<Mutex<Option<ActiveLoaderTask>>>;
static TASKS: LazyLock<Mutex<FxHashMap<u32, TaskCell>>> = LazyLock::new(Default::default);
static WORKERS: LazyLock<Mutex<FxHashMap<u32, Arc<WorkerSlot>>>> = LazyLock::new(Default::default);

#[napi]
pub fn register_loader_worker(id: u32) -> napi::Result<()> {
  // The WASI binding instantiates its own memory in each Node worker. Those
  // workers cannot consume this instance's native task queue.
  if cfg!(target_family = "wasm") {
    return Err(napi::Error::from_reason(
      "Rule.use.parallel requires the native Rspack binding; separate WASI instances do not share the JS task queue",
    ));
  }
  WORKERS
    .lock()
    .expect("loader workers lock poisoned")
    .entry(id)
    .or_default();
  js_task_dispatcher().register_worker(id);
  Ok(())
}

/// Called only after Node reports worker exit: no JS can still access its context.
#[napi]
pub fn stop_loader_worker(id: u32) {
  js_task_dispatcher().stop_worker(id);
  let slot = WORKERS
    .lock()
    .expect("loader workers lock poisoned")
    .remove(&id);
  if let Some(slot) = slot {
    slot.stopped.store(true, Ordering::Release);
    let tasks = std::mem::take(&mut *slot.tasks.lock().expect("loader tasks lock poisoned"));
    for (id, cell) in tasks {
      TASKS
        .lock()
        .expect("loader tasks lock poisoned")
        .remove(&id);
      if let Some(task) = cell.lock().expect("loader task lock poisoned").take() {
        task.finish(
          Vec::new(),
          Err(rspack_error::error!(
            "Parallel loader worker exited before completing its task"
          )),
        );
      }
    }
  }
}

#[napi]
pub struct JsLoaderTask {
  slot: Arc<WorkerSlot>,
  id: u32,
  task: TaskCell,
  context_taken: bool,
  worker: bool,
  pure: bool,
  finished: bool,
}

#[napi]
impl JsLoaderTask {
  #[napi(getter)]
  pub fn id(&self) -> u32 {
    self.id
  }
  #[napi(getter)]
  pub fn pure(&self) -> bool {
    self.pure
  }

  #[napi(getter)]
  pub fn main_object_handle(&self) -> napi::Result<u32> {
    let guard = self.task.lock().expect("loader task lock poisoned");
    Ok(
      guard
        .as_ref()
        .ok_or_else(|| napi::Error::from_reason("Loader task has finished"))?
        .native()
        .main_object_handle,
    )
  }

  #[napi]
  pub fn take_context(&mut self) -> napi::Result<JsLoaderContext> {
    if self.context_taken {
      return Err(napi::Error::from_reason(
        "Loader task context has already been taken",
      ));
    }
    let mut guard = self.task.lock().expect("loader task lock poisoned");
    let task = guard
      .as_mut()
      .ok_or_else(|| napi::Error::from_reason("Loader task has finished"))?;
    if let Some(timing) = &task.native().timing {
      timing.mark(TimingPoint::ContextStarted);
    }
    let mut context = JsLoaderContext::try_from(task.context_mut())
      .map_err(|error| napi::Error::from_reason(error.to_string()))?;
    if self.worker {
      let native = &task.context().context;
      let scope = WorkerModuleScope::new(native.options.context.clone());
      context.module = context
        .module
        .map(|module| module.with_worker_scope(scope.clone()));
      task.module_scope = Some(scope);
    }
    context.loader_chain_start = task.native().start;
    context.loader_chain_end = task.native().end;
    context.bridge_handle = task.bridge_handle.as_ref().map(MainObjectHandle::handle);
    self.context_taken = true;
    if let Some(timing) = &task.native().timing {
      timing.mark(TimingPoint::ContextPrepared);
    }
    Ok(context)
  }

  #[napi]
  pub fn complete(&mut self, context: JsLoaderContext) -> napi::Result<()> {
    let mut task = self
      .task
      .lock()
      .expect("loader task lock poisoned")
      .take()
      .ok_or_else(|| napi::Error::from_reason("Loader task has finished"))?;
    if let Some(timing) = &task.native().timing {
      timing.mark(TimingPoint::CompleteEntered);
    }
    let pitches = context
      .loader_items
      .iter()
      .zip(task.context().loader_items.iter())
      .filter(|(js, _)| js.no_pitch)
      .map(|(_, native)| native.path().to_string())
      .collect();
    let result = merge_loader_context(task.context_mut(), context);
    self.finished = true;
    self.unregister();
    task.finish(pitches, result);
    Ok(())
  }

  #[napi]
  pub fn fail(&mut self, error: String) -> napi::Result<()> {
    let task = self
      .task
      .lock()
      .expect("loader task lock poisoned")
      .take()
      .ok_or_else(|| napi::Error::from_reason("Loader task has finished"))?;
    self.finished = true;
    self.unregister();
    task.finish(Vec::new(), Err(rspack_error::error!(error)));
    Ok(())
  }
}

impl JsLoaderTask {
  pub(crate) fn from_task(
    task: NativeLoaderTask,
    worker: Option<u32>,
    permit: Option<crate::js_tasks::TaskPermit>,
  ) -> Option<Self> {
    let pure = task.mode == LoaderTaskMode::Pure;
    let task = ActiveLoaderTask {
      native: Some(task),
      _permit: permit,
      module_scope: None,
      bridge_handle: None,
    };
    let slot = match worker {
      Some(id) => WORKERS
        .lock()
        .expect("loader workers lock poisoned")
        .get(&id)
        .cloned(),
      None => Some(Arc::default()),
    };
    let Some(slot) = slot else {
      task.finish(
        Vec::new(),
        Err(rspack_error::error!("Loader worker is unavailable")),
      );
      return None;
    };
    let mut active = slot.tasks.lock().expect("loader tasks lock poisoned");
    if slot.stopped.load(Ordering::Acquire) {
      task.finish(
        Vec::new(),
        Err(rspack_error::error!("Loader worker has exited")),
      );
      return None;
    }
    let id = NEXT_TASK.fetch_add(1, Ordering::Relaxed);
    if let Some(timing) = &task.native().timing {
      timing.identify(id, worker);
    }
    let task = Arc::new(Mutex::new(Some(task)));
    active.insert(id, task.clone());
    // Only bridged workers need to look up an active task from the main env.
    if worker.is_some() && !pure {
      TASKS
        .lock()
        .expect("loader tasks lock poisoned")
        .insert(id, task.clone());
    }
    drop(active);
    Some(Self {
      slot,
      id,
      task,
      context_taken: false,
      worker: worker.is_some(),
      pure,
      finished: false,
    })
  }

  pub(crate) fn failure_handle(&self) -> TaskCell {
    self.task.clone()
  }

  pub(crate) fn fail_handle(cell: TaskCell, error: String) {
    if let Some(task) = cell.lock().expect("loader task lock poisoned").take() {
      task.finish(Vec::new(), Err(rspack_error::error!(error)));
    }
  }

  fn unregister(&mut self) {
    self
      .slot
      .tasks
      .lock()
      .expect("loader tasks lock poisoned")
      .remove(&self.id);
    if self.worker && !self.pure {
      TASKS
        .lock()
        .expect("loader tasks lock poisoned")
        .remove(&self.id);
    }
  }
}

impl Drop for JsLoaderTask {
  fn drop(&mut self) {
    self.unregister();
    if !self.finished
      && let Some(task) = self.task.lock().expect("loader task lock poisoned").take()
    {
      task.finish(
        Vec::new(),
        Err(rspack_error::error!("Parallel loader task was abandoned")),
      );
    }
  }
}

/// Main-isolate hook view. Called through RPC after a worker receives native work.
#[napi]
pub fn prepare_loader_task(id: u32) -> napi::Result<JsLoaderContext> {
  let cell = TASKS
    .lock()
    .expect("loader tasks lock poisoned")
    .get(&id)
    .cloned()
    .ok_or_else(|| napi::Error::from_reason("Loader task has finished"))?;
  let mut guard = cell.lock().expect("loader task lock poisoned");
  let task = guard
    .as_mut()
    .ok_or_else(|| napi::Error::from_reason("Loader task has finished"))?;
  let mut context = JsLoaderContext::from_native(task.context_mut(), true)
    .map_err(|error| napi::Error::from_reason(error.to_string()))?;
  context.loader_chain_start = task.native().start;
  context.loader_chain_end = task.native().end;
  Ok(context)
}

#[napi]
pub fn complete_loader_task_hooks(id: u32, context: JsLoaderContext) -> napi::Result<()> {
  let handle = MainObjectHandle::new(
    context
      .bridge_handle
      .ok_or_else(|| napi::Error::from_reason("Loader hooks did not register their context"))?,
  );
  let cell = TASKS
    .lock()
    .expect("loader tasks lock poisoned")
    .get(&id)
    .cloned()
    .ok_or_else(|| napi::Error::from_reason("Loader task has finished"))?;
  let mut guard = cell.lock().expect("loader task lock poisoned");
  let task = guard
    .as_mut()
    .ok_or_else(|| napi::Error::from_reason("Loader task has finished"))?;
  merge_loader_hooks(task.context_mut(), context);
  task.bridge_handle = Some(handle);
  Ok(())
}
