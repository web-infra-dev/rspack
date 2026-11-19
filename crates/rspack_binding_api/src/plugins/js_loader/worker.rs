//! Native loader work is moved between the loader runner and JS isolates. No N-API
//! value is constructed until a worker takes a task's context.
use std::sync::{
  Arc, LazyLock, Mutex,
  atomic::{AtomicBool, AtomicU32, Ordering},
};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{LoaderContext, RunnerContext};
use rustc_hash::FxHashMap;

use super::{JsLoaderContext, result::merge_loader_context};
use crate::{
  error::RspackResultToNapiResultExt,
  js_tasks::{LoaderTask as NativeLoaderTask, TimingPoint, js_task_dispatcher},
  module_bridge::ModuleAccess,
  options::raw_resolve::{
    RawResolveOptionsWithDependencyType, normalize_raw_resolve_options_with_dependency_type,
  },
  resolver::JsResolver,
};

// The shared task contains the native context and is also used on worker exit.
// The live ModuleHandle belongs to the JS-side JsLoaderTask.
pub(crate) struct ActiveLoaderTask {
  native: NativeLoaderTask,
  _completion: Option<tokio_util::sync::DropGuard>,
}

impl ActiveLoaderTask {
  fn context(&self) -> &LoaderContext<RunnerContext> {
    self.native.context()
  }

  fn context_mut(&mut self) -> &mut LoaderContext<RunnerContext> {
    self.native.context_mut()
  }

  fn finish(self, pitches: Vec<String>, result: rspack_error::Result<()>) {
    if let Some(timing) = &self.native.timing {
      timing.mark(TimingPoint::Returned);
    }
    self.native.finish(pitches, result);
  }
}

#[derive(Default)]
struct WorkerSlot {
  stopped: AtomicBool,
  tasks: Mutex<FxHashMap<u32, TaskCell>>,
}

static NEXT_TASK: AtomicU32 = AtomicU32::new(1);
pub(crate) type TaskCell = Arc<Mutex<Option<ActiveLoaderTask>>>;
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
    for cell in tasks.into_values() {
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
  module_access: Option<ModuleAccess>,
  worker: bool,
  finished: bool,
}

#[napi]
impl JsLoaderTask {
  #[napi(getter)]
  pub fn main_object_handle(&self) -> napi::Result<u32> {
    let guard = self.task.lock().expect("loader task lock poisoned");
    Ok(
      guard
        .as_ref()
        .ok_or_else(|| napi::Error::from_reason("Loader task has finished"))?
        .native
        .main_object_handle,
    )
  }

  #[napi(getter)]
  pub fn strict(&self) -> napi::Result<bool> {
    let guard = self.task.lock().expect("loader task lock poisoned");
    Ok(
      guard
        .as_ref()
        .ok_or_else(|| napi::Error::from_reason("Loader task has finished"))?
        .native
        .strict,
    )
  }

  #[napi]
  pub fn get_resolver(
    &self,
    options: Option<RawResolveOptionsWithDependencyType>,
  ) -> napi::Result<JsResolver> {
    let guard = self.task.lock().expect("loader task lock poisoned");
    let task = guard
      .as_ref()
      .ok_or_else(|| napi::Error::from_reason("Loader task has finished"))?;
    let options =
      normalize_raw_resolve_options_with_dependency_type(options, false).to_napi_result()?;
    Ok(JsResolver::new(
      task.context().context.resolver_factory.get(options),
    ))
  }

  #[napi(ts_return_type = "JsLoaderContext")]
  pub fn take_context<'env>(&mut self, env: &'env Env) -> napi::Result<Object<'env>> {
    if self.context_taken {
      return Err(napi::Error::from_reason(
        "Loader task context has already been taken",
      ));
    }
    let mut guard = self.task.lock().expect("loader task lock poisoned");
    let task = guard
      .as_mut()
      .ok_or_else(|| napi::Error::from_reason("Loader task has finished"))?;
    if let Some(timing) = &task.native.timing {
      timing.mark(TimingPoint::ContextStarted);
    }
    if task.native.strict {
      task.context_mut().take_additional_data();
    }
    let mut context = JsLoaderContext::try_from(task.context_mut())
      .map_err(|error| napi::Error::from_reason(error.to_string()))?;
    if let Some(module) = context.module.as_mut() {
      module.worker = self.worker;
    }
    context.loader_chain_start = task.native.start;
    context.loader_chain_end = task.native.end;
    self.context_taken = true;
    if let Some(timing) = &task.native.timing {
      timing.mark(TimingPoint::ContextPrepared);
    }
    drop(guard);
    let (context, access) = context.into_jsobject(env)?;
    self.module_access = access;
    Ok(context)
  }

  #[napi]
  pub fn complete(&mut self, context: JsLoaderContext) -> napi::Result<()> {
    drop(self.module_access.take());
    let mut task = self
      .task
      .lock()
      .expect("loader task lock poisoned")
      .take()
      .ok_or_else(|| napi::Error::from_reason("Loader task has finished"))?;
    if let Some(timing) = &task.native.timing {
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
    drop(self.module_access.take());
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
    completion: Option<tokio_util::sync::DropGuard>,
  ) -> Option<Self> {
    let task = ActiveLoaderTask {
      native: task,
      _completion: completion,
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
    if let Some(timing) = &task.native.timing {
      timing.identify(id, worker);
    }
    let task = Arc::new(Mutex::new(Some(task)));
    active.insert(id, task.clone());
    drop(active);
    Some(Self {
      slot,
      id,
      task,
      context_taken: false,
      module_access: None,
      worker: worker.is_some(),
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
  }
}

impl Drop for JsLoaderTask {
  fn drop(&mut self) {
    drop(self.module_access.take());
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
