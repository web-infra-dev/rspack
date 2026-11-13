use std::{
  ptr,
  sync::{
    Arc, LazyLock, Mutex,
    atomic::{AtomicU32, Ordering},
  },
};

use napi::{bindgen_prelude::*, sys};
use rspack_core::StrictLoaderTask;
use rustc_hash::FxHashMap;
use tokio_util::sync::CancellationToken;

use super::{
  JsLoaderContext,
  scheduler::{collect_loaders_without_pitch, merge_loader_context},
};
use crate::module::WorkerModuleScope;

type ActiveTask = Arc<Mutex<Option<(StrictLoaderTask, Option<Arc<WorkerModuleScope>>)>>>;
type WorkerTasks = FxHashMap<u32, ActiveTask>;

struct Queue {
  sender: async_channel::Sender<StrictLoaderTask>,
  receiver: async_channel::Receiver<StrictLoaderTask>,
  workers: Mutex<FxHashMap<u32, WorkerTasks>>,
}

static QUEUE: LazyLock<Queue> = LazyLock::new(|| {
  let (sender, receiver) = async_channel::unbounded();
  Queue {
    sender,
    receiver,
    workers: Mutex::default(),
  }
});
static NEXT_TASK: AtomicU32 = AtomicU32::new(0);

pub(super) fn dispatch(task: StrictLoaderTask) {
  let workers = QUEUE
    .workers
    .lock()
    .expect("strict loader workers lock poisoned");
  if workers.is_empty() {
    task.finish(
      Vec::new(),
      Err(rspack_error::error!(
        "No strict loader workers are available"
      )),
    );
  } else if let Err(error) = QUEUE.sender.try_send(task) {
    error.into_inner().finish(
      Vec::new(),
      Err(rspack_error::error!("Strict loader queue is closed")),
    );
  }
}

#[napi]
pub fn register_strict_loader_worker(id: u32) {
  QUEUE
    .workers
    .lock()
    .expect("strict loader workers lock poisoned")
    .entry(id)
    .or_default();
}

// The parent calls this after Node reports exit, when JS can no longer access a module.
#[napi]
pub fn stop_strict_loader_worker(id: u32) {
  let tasks = {
    let mut workers = QUEUE
      .workers
      .lock()
      .expect("strict loader workers lock poisoned");
    let tasks = workers.remove(&id);
    if workers.is_empty() {
      while let Ok(task) = QUEUE.receiver.try_recv() {
        task.finish(
          Vec::new(),
          Err(rspack_error::error!(
            "No strict loader workers are available"
          )),
        );
      }
    }
    tasks
  };
  for (_, task) in tasks.into_iter().flatten() {
    finish_failed(&task, "Strict loader worker exited");
  }
}

fn finish_failed(cell: &ActiveTask, reason: &str) {
  if let Some((task, scope)) = cell
    .lock()
    .expect("strict loader task lock poisoned")
    .take()
  {
    if let Some(scope) = scope {
      scope.revoke();
    }
    task.finish(Vec::new(), Err(rspack_error::error!("{reason}")));
  }
}

#[napi]
pub struct JsStrictLoaderTask {
  worker: u32,
  id: u32,
  task: ActiveTask,
  ready: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
}

#[napi]
impl JsStrictLoaderTask {
  #[napi(getter)]
  pub fn owner(&self) -> napi::Result<u32> {
    self
      .task
      .lock()
      .expect("strict loader task lock poisoned")
      .as_ref()
      .map(|(task, _)| task.owner)
      .ok_or_else(|| napi::Error::from_reason("Strict loader task has finished"))
  }

  #[napi]
  pub fn take_context(&self) -> napi::Result<JsLoaderContext> {
    let mut active = self.task.lock().expect("strict loader task lock poisoned");
    let (task, scope) = active
      .as_mut()
      .ok_or_else(|| napi::Error::from_reason("Strict loader task has finished"))?;
    if scope.is_some() {
      return Err(napi::Error::from_reason(
        "Strict loader context has already been taken",
      ));
    }
    let (start, end) = (task.start, task.end);
    let native = task.context_mut();
    let worker_scope = WorkerModuleScope::new(native.context.options.context.clone());
    let mut context = JsLoaderContext::try_from(native)
      .map_err(|error| napi::Error::from_reason(error.to_string()))?;
    context.module = context.module.with_worker_scope(worker_scope.clone());
    *scope = Some(worker_scope);
    context.loader_chain_start = start;
    context.loader_chain_end = end;
    Ok(context)
  }

  #[napi]
  pub fn complete(&self, mut context: JsLoaderContext) -> napi::Result<()> {
    let (mut task, scope) = self
      .task
      .lock()
      .expect("strict loader task lock poisoned")
      .take()
      .ok_or_else(|| napi::Error::from_reason("Strict loader task has finished"))?;
    if let Some(scope) = scope {
      scope.revoke();
    }
    context.additional_data = None;
    let native = task.context_mut();
    let pitches = collect_loaders_without_pitch(native, &context);
    let result = merge_loader_context(native, context);
    task.finish(pitches, result);
    self.unregister();
    Ok(())
  }

  #[napi]
  pub fn fail(&self, reason: String) {
    finish_failed(&self.task, &reason);
    self.unregister();
  }
}

impl JsStrictLoaderTask {
  fn unregister(&self) {
    self
      .ready
      .lock()
      .expect("strict loader permit lock poisoned")
      .take();
    if let Some(tasks) = QUEUE
      .workers
      .lock()
      .expect("strict loader workers lock poisoned")
      .get_mut(&self.worker)
    {
      tasks.remove(&self.id);
    }
  }
}

impl Drop for JsStrictLoaderTask {
  fn drop(&mut self) {
    finish_failed(&self.task, "Strict loader task was abandoned");
    self.unregister();
  }
}

struct Delivery {
  worker: u32,
  task: StrictLoaderTask,
  ready: tokio::sync::oneshot::Sender<()>,
}

struct WorkerCallback(Mutex<sys::napi_threadsafe_function>);

// Node-API allows calling and releasing a TSFN from any native thread.
// The lock serializes posting with environment teardown.
unsafe impl Send for WorkerCallback {}
unsafe impl Sync for WorkerCallback {}

impl WorkerCallback {
  fn post(&self, delivery: Delivery) {
    let callback = self.0.lock().expect("strict loader callback lock poisoned");
    if callback.is_null() {
      return;
    }
    let data = Box::into_raw(Box::new(delivery));
    let status = unsafe {
      sys::napi_call_threadsafe_function(
        *callback,
        data.cast(),
        sys::ThreadsafeFunctionCallMode::nonblocking,
      )
    };
    if status != sys::Status::napi_ok {
      drop(unsafe { Box::from_raw(data) });
    }
  }

  fn abort(&self) {
    let mut callback = self.0.lock().expect("strict loader callback lock poisoned");
    if !callback.is_null() {
      unsafe {
        sys::napi_release_threadsafe_function(*callback, sys::ThreadsafeFunctionReleaseMode::abort)
      };
      *callback = ptr::null_mut();
    }
  }
}

impl Drop for WorkerCallback {
  fn drop(&mut self) {
    self.abort();
  }
}

#[napi]
pub fn subscribe_strict_loader_worker(
  env: Env,
  worker: u32,
  callback: Function<JsStrictLoaderTask, ()>,
) -> napi::Result<()> {
  if cfg!(target_family = "wasm") {
    return Err(napi::Error::from_reason(
      "Strict loader workers require the native binding",
    ));
  }
  rspack_napi::runtime::ensure_runtime(&env)?;
  let mut tsfn = ptr::null_mut();
  let name = env.create_string("rspack:strict-loader")?;
  check_status!(unsafe {
    sys::napi_create_threadsafe_function(
      env.raw(),
      callback.raw(),
      ptr::null_mut(),
      name.raw(),
      0,
      1,
      ptr::null_mut(),
      None,
      ptr::null_mut(),
      Some(call_js),
      &mut tsfn,
    )
  })?;
  let callback = Arc::new(WorkerCallback(Mutex::new(tsfn)));
  check_status!(unsafe { sys::napi_unref_threadsafe_function(env.raw(), tsfn) })?;
  let cancel = CancellationToken::new();
  let cleanup = callback.clone();
  let cleanup_cancel = cancel.clone();
  env.add_env_cleanup_hook((), move |_| {
    cleanup_cancel.cancel();
    cleanup.abort();
    stop_strict_loader_worker(worker);
  })?;
  register_strict_loader_worker(worker);
  rspack_napi::runtime::spawn(async move {
    loop {
      let task = tokio::select! {
        biased;
        _ = cancel.cancelled() => break,
        task = QUEUE.receiver.recv() => task.expect("strict loader queue remains open"),
      };
      let (ready, done) = tokio::sync::oneshot::channel();
      callback.post(Delivery {
        worker,
        task,
        ready,
      });
      tokio::select! {
        biased;
        _ = cancel.cancelled() => break,
        _ = done => {},
      }
    }
  });
  Ok(())
}

extern "C" fn call_js(
  env: sys::napi_env,
  callback: sys::napi_value,
  _context: *mut std::ffi::c_void,
  data: *mut std::ffi::c_void,
) {
  let Delivery {
    worker,
    task,
    ready,
  } = *unsafe { Box::<Delivery>::from_raw(data.cast()) };
  if env.is_null() {
    return;
  }
  let id = NEXT_TASK.fetch_add(1, Ordering::Relaxed);
  let cell = Arc::new(Mutex::new(Some((task, None))));
  {
    let mut workers = QUEUE
      .workers
      .lock()
      .expect("strict loader workers lock poisoned");
    let Some(tasks) = workers.get_mut(&worker) else {
      return;
    };
    tasks.insert(id, cell.clone());
  }
  let task = JsStrictLoaderTask {
    worker,
    id,
    task: cell.clone(),
    ready: Mutex::new(Some(ready)),
  };
  let result = (|| {
    let task = unsafe { ToNapiValue::to_napi_value(env, task) }?;
    let mut receiver = ptr::null_mut();
    check_status!(unsafe { sys::napi_get_undefined(env, &mut receiver) })?;
    let mut ignored = ptr::null_mut();
    check_status!(unsafe {
      sys::napi_call_function(env, receiver, callback, 1, &task, &mut ignored)
    })
  })();
  if let Err(error) = result {
    finish_failed(&cell, &error.to_string());
    let mut exception = ptr::null_mut();
    unsafe { sys::napi_get_and_clear_last_exception(env, &mut exception) };
  }
}
