use std::sync::LazyLock;

use futures::future::BoxFuture;
use tokio::sync::{mpsc, oneshot};

/// TaskQueue manages background async tasks efficiently.
///
/// Tasks are executed sequentially in the order they are added.
/// Uses tokio's unbounded_channel which automatically suspends the receiver when idle.
pub struct TaskQueue {
  sender: LazyLock<mpsc::UnboundedSender<BoxFuture<'static, ()>>>,
}

impl std::fmt::Debug for TaskQueue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "TaskQueue {{ ... }}")
  }
}

impl Default for TaskQueue {
  fn default() -> Self {
    Self::new()
  }
}

impl TaskQueue {
  pub fn new() -> Self {
    TaskQueue {
      sender: LazyLock::new(|| {
        let (tx, mut rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
          while let Some(future) = rx.recv().await {
            future.await
          }
        });

        tx
      }),
    }
  }

  /// Add a task to the queue for sequential execution
  pub fn add_task<F>(&self, task: F)
  where
    F: futures::Future<Output = ()> + Send + 'static,
  {
    // Send is synchronous and cheap - just adds to channel
    let _ = self.sender.send(Box::pin(task));
  }

  /// Wait for all pending tasks to complete
  pub async fn flush(&self) {
    let (tx, rx) = oneshot::channel();

    // Add a flush task that signals completion
    self.add_task(async move {
      let _ = tx.send(());
    });

    // Wait for the flush task to complete
    let _ = rx.await;
  }
}
