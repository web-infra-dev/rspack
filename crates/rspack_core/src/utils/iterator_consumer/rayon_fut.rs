use std::{future::Future, pin::Pin, sync::Arc};

use rayon::iter::ParallelIterator;
use tokio::sync::mpsc::unbounded_channel;

use super::{RayonConsumer, bound_future::BoundFuture};

/// Tools for consume rayon iterator which return feature.
pub trait RayonFutureConsumer<'a> {
  type Item;
  /// Drives every future in the parallel iterator concurrently via `tokio::spawn`
  /// and calls `func` with each output on the current thread as results arrive.
  /// The returned [`BoundFuture`] must be awaited to completion.
  fn fut_consume(self, func: impl FnMut(Self::Item) + Send + 'a) -> BoundFuture<'a, ()>;
}

impl<'a, I, Fut> RayonFutureConsumer<'a> for I
where
  I: ParallelIterator<Item = Fut> + Send + 'a,
  Fut: Future + Send + 'a,
  Fut::Output: Send + 'a,
{
  type Item = Fut::Output;

  fn fut_consume(self, mut func: impl FnMut(Self::Item) + Send + 'a) -> BoundFuture<'a, ()> {
    BoundFuture::new(async move {
      let (tx, mut rx) = unbounded_channel::<Fut::Output>();
      let tx = Arc::new(tx);

      // `consume` drives the parallel iterator on the rayon thread pool and
      // delivers futures to the closure on the calling thread (blocking).
      self.consume(|fut| {
        let tx = tx.clone();

        let task: Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> =
          Box::pin(async move {
            tx.send(fut.await).expect("should send success");
          });

        // SAFETY: same argument as in `FutureConsumer::fut_consume`.
        let static_task: Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>> =
          unsafe { std::mem::transmute(task) };
        tokio::spawn(static_task);
      });

      // Drop the last sender so the channel closes after the iterator is exhausted.
      drop(tx);

      while let Some(data) = rx.recv().await {
        func(data);
      }
    })
  }
}

#[cfg(test)]
mod test {
  use rayon::prelude::*;

  use super::RayonFutureConsumer;

  #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
  async fn available() {
    (0..10)
      .into_par_iter()
      .map(|item| async move { item * 2 })
      .fut_consume(|item| assert_eq!(item % 2, 0))
      .await;
  }
}
