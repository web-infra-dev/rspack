use std::sync::{Arc, Mutex};

use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

#[derive(Default)]
struct State {
  active: usize,
  imports: usize,
}

/// A worker normally consumes one task. An unresolved import allows one more
/// task so a module can recursively build on the same worker.
#[derive(Default)]
pub(crate) struct WorkerCapacity {
  state: Mutex<State>,
  pub(crate) changed: Notify,
}

impl WorkerCapacity {
  pub(crate) async fn acquire(self: &Arc<Self>, cancel: &CancellationToken) -> Option<TaskPermit> {
    loop {
      let changed = self.changed.notified();
      tokio::pin!(changed);
      changed.as_mut().enable();
      {
        let mut state = self.state.lock().expect("worker capacity lock poisoned");
        if state.active < 1 + state.imports {
          state.active += 1;
          return Some(TaskPermit(self.clone()));
        }
      }
      tokio::select! {
        biased;
        _ = cancel.cancelled() => return None,
        _ = changed => {},
      }
    }
  }

  pub(crate) fn begin_import(self: &Arc<Self>) -> ImportPermit {
    self
      .state
      .lock()
      .expect("worker capacity lock poisoned")
      .imports += 1;
    self.changed.notify_waiters();
    ImportPermit(self.clone())
  }
}

pub(crate) struct TaskPermit(Arc<WorkerCapacity>);

impl Drop for TaskPermit {
  fn drop(&mut self) {
    self
      .0
      .state
      .lock()
      .expect("worker capacity lock poisoned")
      .active -= 1;
    self.0.changed.notify_waiters();
  }
}

pub struct ImportPermit(Arc<WorkerCapacity>);

impl Drop for ImportPermit {
  fn drop(&mut self) {
    self
      .0
      .state
      .lock()
      .expect("worker capacity lock poisoned")
      .imports -= 1;
    self.0.changed.notify_waiters();
  }
}
