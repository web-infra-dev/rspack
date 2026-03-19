use std::{future::Future, pin::Pin, sync::Arc};

use tokio::sync::mpsc::unbounded_channel;

use super::bound_future::BoundFuture;

/// Tools for consume iterator which return future.
pub trait FutureConsumer<'a> {
  type Item;
  /// Drives every future in the iterator concurrently via `tokio::spawn` and
  /// calls `func` with each output on the current thread as results arrive.
  /// The returned [`BoundFuture`] must be awaited to completion.
  fn fut_consume(self, func: impl FnMut(Self::Item) + Send + 'a) -> BoundFuture<'a, ()>;
}

impl<'a, I, Fut> FutureConsumer<'a> for I
where
  I: Iterator<Item = Fut> + Send + 'a,
  Fut: Future + Send + 'a,
  Fut::Output: Send + 'a,
{
  type Item = Fut::Output;

  fn fut_consume(self, mut func: impl FnMut(Self::Item) + Send + 'a) -> BoundFuture<'a, ()> {
    BoundFuture::new(async move {
      let (tx, mut rx) = unbounded_channel::<Fut::Output>();
      let tx = Arc::new(tx);

      self.for_each(|fut| {
        let tx = tx.clone();

        let task: Pin<Box<dyn Future<Output = ()> + Send + 'a>> = Box::pin(async move {
          tx.send(fut.await).expect("should send success");
        });

        // SAFETY: We transmute the task from `'a` to `'static` to satisfy
        // `tokio::spawn`. This is sound because:
        // 1. Spawning only occurs while this async block is being polled, so
        //    no task is spawned if `BoundFuture` is dropped before first poll.
        // 2. `BoundFuture::drop` panics unless `completed` is true, which
        //    requires polling to `Ready`. The future is `Ready` only after
        //    `rx` is drained, which happens only after every task has sent
        //    its result and exited — so no task outlives `'a`.
        // 3. `Fut::Output` may borrow from `'a`; the task sends it into the
        //    channel immediately and exits, and the channel buffer is dropped
        //    within `'a`, so no `'a` data escapes.
        let static_task: Pin<Box<dyn Future<Output = ()> + Send + 'static>> =
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
  use super::FutureConsumer;

  #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
  async fn available() {
    (0..10)
      .map(|item| async move { item * 2 })
      .fut_consume(|item| assert_eq!(item % 2, 0))
      .await;
  }
}
