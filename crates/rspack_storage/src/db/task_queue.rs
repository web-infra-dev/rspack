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
}

impl TaskQueue {
  /// Add a task to the queue for sequential execution
  pub fn add_task(&self, task: impl Future<Output = ()> + Send + 'static) {
    self.sender.send(Box::pin(task)).expect("should add task");
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

#[cfg(test)]
mod tests {
  use std::{sync::Arc, time::Duration};

  use tokio::sync::{Mutex, oneshot};

  use super::TaskQueue;

  async fn test_task_queue() {
    let queue = TaskQueue::default();
    let (tx_0, rx_0) = oneshot::channel();
    let (tx_1, rx_1) = oneshot::channel();
    let (tx_2, rx_2) = oneshot::channel();

    let inc = Arc::new(Mutex::new(0_usize));

    let inc_0 = inc.clone();
    queue.add_task(Box::pin(async move {
      tokio::time::sleep(Duration::from_millis(30)).await;
      let mut inc = inc_0.lock().await;
      *inc += 1;
      tx_0.send(*inc).unwrap();
    }));
    let inc_1 = inc.clone();
    queue.add_task(Box::pin(async move {
      tokio::time::sleep(Duration::from_millis(20)).await;
      let mut inc = inc_1.lock().await;
      *inc += 1;
      tx_1.send(*inc).unwrap();
    }));
    let inc_2 = inc.clone();
    queue.add_task(Box::pin(async move {
      tokio::time::sleep(Duration::from_millis(10)).await;
      let mut inc = inc_2.lock().await;
      *inc += 1;
      tx_2.send(*inc).unwrap();
    }));

    assert_eq!(rx_0.await, Ok(1));
    assert_eq!(rx_1.await, Ok(2));
    assert_eq!(rx_2.await, Ok(3));
  }

  #[test]
  #[cfg_attr(miri, ignore)]
  fn should_add_task_to_queue() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
      test_task_queue().await;
    });
  }
}
