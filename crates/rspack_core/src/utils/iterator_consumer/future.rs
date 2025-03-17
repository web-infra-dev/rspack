use std::future::Future;
use std::sync::Arc;

use tokio::sync::mpsc::unbounded_channel;

/// Tools for consume iterator which return future.
#[async_trait::async_trait]
pub trait FutureConsumer {
  type Item;
  /// Use to immediately consume the data produced by the future in the iterator
  /// without waiting for all the data to be processed.
  /// The closures runs in the current thread.
  async fn fut_consume(self, func: impl FnMut(Self::Item) + Send);
}

#[async_trait::async_trait]
impl<I, Fut> FutureConsumer for I
where
  I: Iterator<Item = Fut> + Send,
  Fut: Future + Send + 'static,
  Fut::Output: Send + 'static,
{
  type Item = Fut::Output;
  async fn fut_consume(self, mut func: impl FnMut(Self::Item) + Send) {
    let mut rx = {
      // Create the channel in the closure to ensure all sender are dropped when iterator completes
      // This ensures that the receiver does not get stuck in an infinite loop.
      let (tx, rx) = unbounded_channel::<Self::Item>();
      let tx = Arc::new(tx);
      self.for_each(|fut| {
        let tx = tx.clone();
        tokio::spawn(async move {
          let data = fut.await;
          tx.send(data).expect("should send success");
        });
      });
      rx
    };

    while let Some(data) = rx.recv().await {
      func(data);
    }
  }
}

#[cfg(test)]
mod test {
  use std::time::SystemTime;

  use futures::future::join_all;
  use tokio::time::{sleep, Duration};

  use super::FutureConsumer;

  #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
  async fn available() {
    (0..10)
      .map(|item| async move { item * 2 })
      .fut_consume(|item| assert_eq!(item % 2, 0))
      .await;
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
  async fn time_check() {
    let start = SystemTime::now();
    vec![100, 200]
      .into_iter()
      .map(|item| async move {
        sleep(Duration::from_millis(item)).await;
        item
      })
      .fut_consume(|_| {
        std::thread::sleep(std::time::Duration::from_millis(20));
      })
      .await;
    let time1 = SystemTime::now().duration_since(start).unwrap();

    let start = SystemTime::now();
    let data = join_all(vec![100, 200].into_iter().map(|item| async move {
      sleep(Duration::from_millis(item)).await;
      item
    }))
    .await;
    for _ in data.iter() {
      sleep(Duration::from_millis(20)).await;
    }
    let time2 = SystemTime::now().duration_since(start).unwrap();
    assert!(time1 < time2);
  }
}
