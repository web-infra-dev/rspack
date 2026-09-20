//! Native loader work is moved between the scheduler and JS isolates. No N-API
//! value is constructed until a worker takes a task's context.
use std::sync::{
  Arc, LazyLock, Mutex,
  atomic::{AtomicBool, AtomicU32, Ordering},
};

use async_channel::{Receiver, Sender};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{LoaderContext, RunnerContext};
use rustc_hash::FxHashMap;
use tokio::sync::{Notify, oneshot};

use super::{
  JsLoaderContext,
  bridge::MainObjectHandle,
  scheduler::{merge_loader_context, merge_loader_hooks},
};

type TaskResult = (
  Box<LoaderContext<RunnerContext>>,
  Vec<String>,
  rspack_error::Result<()>,
);

pub(super) struct LoaderTask {
  pub context: Box<LoaderContext<RunnerContext>>,
  pub bridge_handle: Option<MainObjectHandle>,
  pub main_object_handle: u32,
  pub start: u32,
  pub end: u32,
  pub result: oneshot::Sender<TaskResult>,
}

impl LoaderTask {
  fn finish(self, pitches: Vec<String>, result: rspack_error::Result<()>) {
    let _ = self.result.send((self.context, pitches, result));
  }
}

#[derive(Default)]
struct WorkerSlot {
  stopped: AtomicBool,
  notify: Notify,
  tasks: Mutex<FxHashMap<u32, TaskCell>>,
}

static QUEUE: LazyLock<(Sender<LoaderTask>, Receiver<LoaderTask>)> =
  LazyLock::new(async_channel::unbounded);
static NEXT_TASK: AtomicU32 = AtomicU32::new(1);
type TaskCell = Arc<Mutex<Option<LoaderTask>>>;
static TASKS: LazyLock<Mutex<FxHashMap<u32, TaskCell>>> = LazyLock::new(Default::default);
#[derive(Default)]
struct ReceiveState {
  cancelled: AtomicBool,
  notify: Notify,
}
type ReceiverMap = FxHashMap<(u32, u32), Arc<ReceiveState>>;
static RECEIVERS: LazyLock<Mutex<ReceiverMap>> = LazyLock::new(Default::default);
static WORKERS: LazyLock<Mutex<FxHashMap<u32, Arc<WorkerSlot>>>> = LazyLock::new(Default::default);

pub(super) async fn dispatch(task: LoaderTask) {
  let workers = WORKERS.lock().expect("loader workers lock poisoned");
  if workers.is_empty() {
    task.finish(
      Vec::new(),
      Err(rspack_error::error!(
        "No parallel loader workers are available"
      )),
    );
    return;
  }
  // The unbounded channel never waits for capacity. Keep registration and
  // enqueue atomic with respect to retiring the last worker.
  if let Err(error) = QUEUE.0.try_send(task) {
    error.into_inner().finish(
      Vec::new(),
      Err(rspack_error::error!("Parallel loader queue is closed")),
    );
  }
}

#[napi]
pub fn register_loader_worker(id: u32) {
  WORKERS
    .lock()
    .expect("loader workers lock poisoned")
    .insert(id, Arc::default());
}

/// Called only after Node reports worker exit: no JS can still access its context.
#[napi]
pub fn stop_loader_worker(id: u32) {
  let (slot, empty) = {
    let mut workers = WORKERS.lock().expect("loader workers lock poisoned");
    let slot = workers.remove(&id);
    (slot, workers.is_empty())
  };
  if let Some(slot) = slot {
    slot.stopped.store(true, Ordering::Release);
    slot.notify.notify_waiters();
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
  if empty {
    while let Ok(task) = QUEUE.1.try_recv() {
      task.finish(
        Vec::new(),
        Err(rspack_error::error!(
          "No parallel loader workers are available"
        )),
      );
    }
  }
}

#[napi]
pub struct JsLoaderTask {
  slot: Arc<WorkerSlot>,
  id: u32,
  task: TaskCell,
  context_taken: bool,
  finished: bool,
}

#[napi]
impl JsLoaderTask {
  #[napi(getter)]
  pub fn id(&self) -> u32 {
    self.id
  }
  #[napi(getter)]
  pub fn main_object_handle(&self) -> napi::Result<u32> {
    let guard = self.task.lock().expect("loader task lock poisoned");
    Ok(
      guard
        .as_ref()
        .ok_or_else(|| napi::Error::from_reason("Loader task has finished"))?
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
    let mut context = JsLoaderContext::try_from(task.context.as_mut())
      .map_err(|error| napi::Error::from_reason(error.to_string()))?;
    // Native-backed modules stay in the compiler isolate; workers use the RPC proxy.
    context.module = None;
    context.loader_chain_start = task.start;
    context.loader_chain_end = task.end;
    context.bridge_handle = task.bridge_handle.as_ref().map(MainObjectHandle::handle);
    self.context_taken = true;
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
    let pitches = context
      .loader_items
      .iter()
      .zip(task.context.loader_items.iter())
      .filter(|(js, _)| js.no_pitch)
      .map(|(_, native)| native.path().to_string())
      .collect();
    let result = merge_loader_context(&mut task.context, context);
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
  fn unregister(&self) {
    self
      .slot
      .tasks
      .lock()
      .expect("loader tasks lock poisoned")
      .remove(&self.id);
    TASKS
      .lock()
      .expect("loader tasks lock poisoned")
      .remove(&self.id);
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

#[napi]
pub fn cancel_worker_receive(worker: u32, receive: u32) {
  if let Some(state) = RECEIVERS
    .lock()
    .expect("loader receivers lock poisoned")
    .get(&(worker, receive))
  {
    state.cancelled.store(true, Ordering::Release);
    state.notify.notify_one();
  }
}

#[napi]
pub fn recv_worker_task(
  env: &Env,
  worker: u32,
  receive: u32,
) -> napi::Result<PromiseRaw<'_, Option<JsLoaderTask>>> {
  let slot = WORKERS
    .lock()
    .expect("loader workers lock poisoned")
    .get(&worker)
    .cloned()
    .ok_or_else(|| napi::Error::from_reason("Loader worker is not registered"))?;
  let state = Arc::<ReceiveState>::default();
  RECEIVERS
    .lock()
    .expect("loader receivers lock poisoned")
    .insert((worker, receive), state.clone());
  rspack_napi::runtime::promise_from_future(env, async move {
    let result = async {
      let stopped = slot.notify.notified();
      tokio::pin!(stopped);
      stopped.as_mut().enable();
      if state.cancelled.load(Ordering::Acquire) || slot.stopped.load(Ordering::Acquire) {
        return Ok(None);
      }
      let task = tokio::select! {
        biased;
        _ = stopped => return Ok(None),
        _ = state.notify.notified() => return Ok(None),
        task = QUEUE.1.recv() => task.map_err(|_| napi::Error::from_reason("Parallel loader queue is closed"))?,
      };
      let mut active = slot.tasks.lock().expect("loader tasks lock poisoned");
      // Worker exit may race with channel delivery. Always return its owned
      // context to the scheduler even if no JS wrapper can be created anymore.
      if slot.stopped.load(Ordering::Acquire) {
        task.finish(Vec::new(), Err(rspack_error::error!("Parallel loader worker exited before completing its task")));
        return Ok(None);
      }
      let id = NEXT_TASK.fetch_add(1, Ordering::Relaxed);
      let task = Arc::new(Mutex::new(Some(task)));
      active.insert(id, task.clone());
      TASKS.lock().expect("loader tasks lock poisoned").insert(id, task.clone());
      Ok(Some(JsLoaderTask {
        slot: slot.clone(), id, task, context_taken: false, finished: false,
      }))
    }.await;
    RECEIVERS
      .lock()
      .expect("loader receivers lock poisoned")
      .remove(&(worker, receive));
    result
  })
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
  let mut context = JsLoaderContext::from_native(&mut task.context, true)
    .map_err(|error| napi::Error::from_reason(error.to_string()))?;
  context.loader_chain_start = task.start;
  context.loader_chain_end = task.end;
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
  merge_loader_hooks(&mut task.context, context);
  task.bridge_handle = Some(handle);
  Ok(())
}
