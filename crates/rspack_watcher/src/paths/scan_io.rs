use std::{
  panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
  sync::{Mutex, MutexGuard},
};

use futures::{StreamExt, stream};
use tokio::sync::{mpsc, oneshot};

type Read = Box<dyn FnOnce() + Send>;
type Sender = mpsc::Sender<Read>;
pub(crate) const MAX_SCAN_READS: usize = 16;

/// The registration task owns the receiver. Dropping its runtime closes outstanding replies.
#[derive(Default)]
pub(super) struct ScanIo(Mutex<Option<Sender>>);

impl ScanIo {
  fn sender(&self) -> MutexGuard<'_, Option<Sender>> {
    self.0.lock().expect("scan mailbox poisoned")
  }

  pub(super) fn start(&self) {
    self.sender().get_or_insert_with(|| {
      let (send, mut jobs) = mpsc::channel::<Read>(MAX_SCAN_READS);
      let reads = stream::poll_fn(move |cx| jobs.poll_recv(cx));
      tokio::spawn(
        reads.for_each_concurrent(MAX_SCAN_READS, |read| async move {
          let _ = tokio::task::spawn_blocking(read).await;
        }),
      );
      send
    });
  }

  pub(super) fn close(&self) {
    self.sender().take();
  }
  pub(super) fn is_open(&self) -> bool {
    self.sender().as_ref().is_some_and(|s| !s.is_closed())
  }

  pub(super) async fn read<T: Send + 'static>(
    &self,
    read: impl FnOnce() -> T + Send + 'static,
  ) -> Option<T> {
    let jobs = self.sender().clone()?;
    let (reply, result) = oneshot::channel();
    jobs
      .send(Box::new(move || {
        if !reply.is_closed() {
          let _ = reply.send(catch_unwind(AssertUnwindSafe(read)));
        }
      }))
      .await
      .ok()?;
    match result.await.ok()? {
      Ok(value) => Some(value),
      Err(panic) => resume_unwind(panic),
    }
  }
}
