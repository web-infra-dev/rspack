use std::{fmt::Debug, future::Future, sync::LazyLock};

use futures::{future::BoxFuture, FutureExt};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

pub struct TaskQueue(LazyLock<UnboundedSender<BoxFuture<'static, ()>>>);

impl Debug for TaskQueue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "TaskQueue {{... }}")
  }
}

impl TaskQueue {
  pub fn new() -> Self {
    TaskQueue(LazyLock::new(|| {
      let (tx, mut rx) = unbounded_channel();
      tokio::spawn(async move {
        while let Some(future) = rx.recv().await {
          future.await
        }
      });

      tx
    }))
  }

  pub fn add_task(&self, task: impl Future<Output = ()> + Send + 'static) {
    self.0.send(task.boxed()).expect("should add task");
  }
}

#[cfg(test)]
mod tests {
  use std::{sync::Arc, time::Duration};

  use tokio::sync::{oneshot, Mutex};

  use crate::pack::manager::queue::TaskQueue;

  async fn test_task_queue() {
    let queue = TaskQueue::new();
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
