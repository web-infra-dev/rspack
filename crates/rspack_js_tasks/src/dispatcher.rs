use std::{
  collections::HashSet,
  sync::{LazyLock, Mutex},
};

use async_channel::{Receiver, Sender};

use crate::{JsTask, TimingPoint};

static JS_TASK_DISPATCHER: LazyLock<JsTaskDispatcher> = LazyLock::new(JsTaskDispatcher::new);

pub fn js_task_dispatcher() -> &'static JsTaskDispatcher {
  &JS_TASK_DISPATCHER
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
    task.mark_timing(TimingPoint::Enqueued);
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

  pub fn try_recv(&self) -> Option<JsTask> {
    self.0.try_recv().ok()
  }
}

pub struct JsTaskDispatcher {
  worker: TaskQueue,
  workers: Mutex<HashSet<u32>>,
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

  pub fn register_worker(&self, id: u32) {
    self
      .workers
      .lock()
      .expect("JS task workers lock poisoned")
      .insert(id);
  }

  /// Retire an exited worker or a failed startup slot. Registration, enqueue and
  /// draining the last consumer's queue use the same lock.
  pub fn stop_worker(&self, id: u32) {
    let mut workers = self.workers.lock().expect("JS task workers lock poisoned");
    workers.remove(&id);
    if workers.is_empty() {
      while let Ok(task) = self.worker.receiver.try_recv() {
        task.fail("No parallel loader workers are available");
      }
    }
  }
}
