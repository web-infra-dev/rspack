use std::future::Future;
use std::sync::Arc;

use futures::future::BoxFuture;
use tokio::sync::mpsc::unbounded_channel;

#[async_trait::async_trait]
pub trait FutureConsumer {
  type Item;
  async fn fut_consume<F>(self, func: impl Fn(Self::Item) -> F + Send)
  where
    F: Future + Send;
}

#[async_trait::async_trait]
impl<I, Fut> FutureConsumer for I
where
  I: Iterator<Item = Fut> + Send,
  Fut: Future + Send,
  Fut::Output: Send + 'static,
{
  type Item = Fut::Output;
  async fn fut_consume<F>(self, func: impl Fn(Self::Item) -> F + Send)
  where
    F: Future + Send,
  {
    let mut rx = {
      let (tx, rx) = unbounded_channel::<Self::Item>();
      let tx = Arc::new(tx);
      self.for_each(|fut| {
        let boxed_fut: BoxFuture<Fut::Output> = Box::pin(fut);
        // SAFETY: We will send results to channel and process all data using func param,
        // therefore all `tokio::spawn` will complete before fut_consume finish, and we can pass any lifecycle fut to `tokio::spawn`.
        // This unsafe transmute is used to workaround the `tokio::spawn` lifecycle check.
        let fut: BoxFuture<'static, Fut::Output> = unsafe { std::mem::transmute(boxed_fut) };
        let tx = tx.clone();
        tokio::spawn(async move {
          let data = fut.await;
          tx.send(data).expect("should send success");
        });
      });
      rx
    };

    while let Some(data) = rx.recv().await {
      func(data).await;
    }
  }
}

#[cfg(test)]
mod test {
  use std::time::SystemTime;

  use rspack_futures::FuturesResults;
  use tokio::time::{sleep, Duration};

  use super::FutureConsumer;

  #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
  async fn available() {
    (0..10)
      .into_iter()
      .map(|item| async move { item * 2 })
      .fut_consume(|item| async move { assert_eq!(item % 2, 0) })
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
      .fut_consume(|_| async move {
        sleep(Duration::from_millis(20)).await;
      })
      .await;
    let time1 = SystemTime::now().duration_since(start).unwrap();

    let start = SystemTime::now();
    let data = vec![100, 200]
      .into_iter()
      .map(|item| async move {
        sleep(Duration::from_millis(item)).await;
        item
      })
      .collect::<FuturesResults<_>>();
    for _ in data.into_inner() {
      sleep(Duration::from_millis(20)).await;
    }
    let time2 = SystemTime::now().duration_since(start).unwrap();
    assert!(time1 < time2);
  }
}
