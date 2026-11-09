use std::{
  collections::HashSet,
  sync::{Arc, LazyLock, Mutex, Weak},
};

use async_channel::{Receiver, Sender};
use tokio_util::sync::CancellationToken;

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

/// The main subscription owns this registration. Dispatch borrows it only
/// until enqueue; neither the global queue nor its receive loop keeps it alive.
pub struct MainTaskRegistration {
  active: Mutex<bool>,
  ready: CancellationToken,
  cancel: CancellationToken,
}

impl MainTaskRegistration {
  pub fn stop(&self) {
    *self
      .active
      .lock()
      .expect("main task registration lock poisoned") = false;
    self.cancel.cancel();
  }
}

impl Drop for MainTaskRegistration {
  fn drop(&mut self) {
    self.stop();
  }
}

pub struct JsTaskDispatcher {
  main: TaskQueue,
  worker: TaskQueue,
  main_registration: Mutex<Weak<MainTaskRegistration>>,
  workers: Mutex<HashSet<u32>>,
}

impl JsTaskDispatcher {
  fn new() -> Self {
    Self {
      main: TaskQueue::new(),
      worker: TaskQueue::new(),
      main_registration: Mutex::new(Weak::new()),
      workers: Mutex::new(HashSet::new()),
    }
  }

  pub fn register_main(
    &self,
    ready: CancellationToken,
    cancel: CancellationToken,
  ) -> Arc<MainTaskRegistration> {
    let registration = Arc::new(MainTaskRegistration {
      active: Mutex::new(true),
      ready,
      cancel,
    });
    *self
      .main_registration
      .lock()
      .expect("main task registration lock poisoned") = Arc::downgrade(&registration);
    registration
  }

  pub fn subscribe_main(&self) -> JsTaskReceiver {
    self.main.subscribe()
  }
  pub fn subscribe_worker(&self) -> JsTaskReceiver {
    self.worker.subscribe()
  }

  pub async fn dispatch_main(&self, task: JsTask) {
    let registration = self
      .main_registration
      .lock()
      .expect("main task registration lock poisoned")
      .upgrade();
    let Some(registration) = registration else {
      task.fail("Main JS task subscription is unavailable");
      return;
    };
    tokio::select! {
      biased;
      _ = registration.cancel.cancelled() => {
        task.fail("Main JS task subscription has closed");
        return;
      }
      _ = registration.ready.cancelled() => {},
    }
    // Serialize cancellation with enqueue so a stopped loop cannot leave new
    // work behind after draining its queue.
    let active = registration
      .active
      .lock()
      .expect("main task registration lock poisoned");
    if *active {
      self.main.dispatch(task);
    } else {
      task.fail("Main JS task subscription has closed");
    }
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
