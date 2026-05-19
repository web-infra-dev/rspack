use std::{future::Future, sync::Arc};

use rspack_tasks::spawn_in_context;
use tokio::{sync::mpsc::unbounded_channel, task::JoinHandle};

/// Like [`FutureConsumer`](super::future::FutureConsumer), but for fallible futures.
///
/// Spawns all futures concurrently. Calls `func` for each `Ok` value as it
/// arrives. On the first `Err`, cancels remaining tasks and returns that error.
pub trait TryFutureConsumer {
  type Ok;
  type Err;
  fn try_fut_consume(
    self,
    func: impl FnMut(Self::Ok) + Send,
  ) -> impl Future<Output = Result<(), Self::Err>>;
}

impl<I, Fut, T, E> TryFutureConsumer for I
where
  I: Iterator<Item = Fut>,
  Fut: Future<Output = Result<T, E>> + Send + 'static,
  T: Send + 'static,
  E: Send + 'static,
{
  type Ok = T;
  type Err = E;

  fn try_fut_consume(
    self,
    mut func: impl FnMut(Self::Ok) + Send,
  ) -> impl Future<Output = Result<(), Self::Err>> {
    let (tx, rx) = unbounded_channel::<Result<T, E>>();
    let tx = Arc::new(tx);
    let handles: Vec<JoinHandle<()>> = self
      .map(|fut| {
        let tx = tx.clone();
        spawn_in_context(async move {
          let data = fut.await;
          // Ignore send errors: the channel may already be closed.
          tx.send(data).ok();
        })
      })
      .collect();
    // Drop our copy so the channel closes once all tasks finish.
    drop(tx);

    async move {
      let mut rx = rx;
      while let Some(result) = rx.recv().await {
        match result {
          Ok(v) => func(v),
          Err(e) => {
            rx.close();
            // Best-effort: abort tasks still waiting at an .await point.
            for handle in &handles {
              handle.abort();
            }
            return Err(e);
          }
        }
      }
      Ok(())
    }
  }
}

#[cfg(test)]
mod test {
  use super::TryFutureConsumer;

  #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
  async fn try_available() {
    let result: Result<(), &str> = (0..10)
      .map(|item| async move { Ok::<_, &str>(item * 2) })
      .try_fut_consume(|item| assert_eq!(item % 2, 0))
      .await;
    assert!(result.is_ok());
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
  async fn try_short_circuits_on_error() {
    let result: Result<(), &str> = (0..10)
      .map(|item| async move { if item == 5 { Err("boom") } else { Ok(item) } })
      .try_fut_consume(|_| {})
      .await;
    assert_eq!(result, Err("boom"));
  }
}
