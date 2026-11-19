use std::{
  collections::HashSet,
  sync::{Arc, LazyLock, Mutex, Weak},
};

use async_channel::{Receiver, Sender};
use rustc_hash::FxHashMap;

use super::JsTask;

/// Node `worker_threads` thread id used to route loader tasks to their owner.
pub(crate) type WorkerThreadId = u32;

thread_local! {
  // One queue per JS env thread; compilers constructed by the same env share it.
  static JS_TASK_DISPATCHER: Arc<JsTaskDispatcher> = Arc::new(JsTaskDispatcher::new());
}

pub(crate) fn js_task_dispatcher() -> Arc<JsTaskDispatcher> {
  JS_TASK_DISPATCHER.with(Arc::clone)
}

/// Worker envs cannot read the owner env's thread-local queue. The main env
/// registers each worker thread id here so its subscription can attach to the
/// owning queue. This is lookup-only state: multiple owners may coexist.
static WORKER_DISPATCHERS: LazyLock<Mutex<FxHashMap<WorkerThreadId, Weak<JsTaskDispatcher>>>> =
  LazyLock::new(Default::default);

pub(crate) fn register_worker_dispatcher(id: WorkerThreadId, dispatcher: &Arc<JsTaskDispatcher>) {
  WORKER_DISPATCHERS
    .lock()
    .expect("worker dispatchers lock poisoned")
    .insert(id, Arc::downgrade(dispatcher));
}

pub(crate) fn worker_dispatcher(id: WorkerThreadId) -> Option<Arc<JsTaskDispatcher>> {
  WORKER_DISPATCHERS
    .lock()
    .expect("worker dispatchers lock poisoned")
    .get(&id)
    .and_then(Weak::upgrade)
}

pub(crate) fn unregister_worker_dispatcher(id: u32) {
  WORKER_DISPATCHERS
    .lock()
    .expect("worker dispatchers lock poisoned")
    .remove(&id);
}

struct TaskQueue {
  sender: Sender<JsTask>,
  receiver: Receiver<JsTask>,
}

impl TaskQueue {
  fn new() -> Self {
    let (sender, receiver) = async_channel::unbounded();
    Self { sender, receiver }
  }

  fn dispatch(&self, task: JsTask) {
    if let Err(error) = self.sender.try_send(task) {
      error.into_inner().fail("JS task queue is closed");
    }
  }

  fn subscribe(&self) -> JsTaskReceiver {
    JsTaskReceiver(self.receiver.clone())
  }
}

/// A receiver cannot close the shared channel or access its sender.
pub struct JsTaskReceiver(Receiver<JsTask>);

impl JsTaskReceiver {
  pub async fn recv(&self) -> JsTask {
    self
      .0
      .recv()
      .await
      .expect("global JS task queues remain open")
  }
}

pub struct JsTaskDispatcher {
  worker: TaskQueue,
  workers: Mutex<HashSet<WorkerThreadId>>,
}

impl JsTaskDispatcher {
  fn new() -> Self {
    Self {
      worker: TaskQueue::new(),
      workers: Mutex::new(HashSet::new()),
    }
  }

  pub fn subscribe_worker(&self) -> JsTaskReceiver {
    self.worker.subscribe()
  }

  pub fn dispatch_worker(&self, task: JsTask) {
    let workers = self.workers.lock().expect("JS task workers lock poisoned");
    if workers.is_empty() {
      task.fail("No parallel loader workers are available");
    } else {
      self.worker.dispatch(task);
    }
  }

  pub fn register_worker(&self, id: WorkerThreadId) {
    self
      .workers
      .lock()
      .expect("JS task workers lock poisoned")
      .insert(id);
  }

  /// Retire an exited worker or a failed startup slot. Registration, enqueue and
  /// draining the last consumer's queue use the same lock.
  pub fn stop_worker(&self, id: WorkerThreadId) {
    let mut workers = self.workers.lock().expect("JS task workers lock poisoned");
    workers.remove(&id);
    if workers.is_empty() {
      while let Ok(task) = self.worker.receiver.try_recv() {
        task.fail("No parallel loader workers are available");
      }
    }
  }
}
