use std::{
  error::Error,
  fmt::{self, Display, Formatter},
  sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
  },
  time::{Duration, Instant},
};

use async_channel::{Receiver, Sender};
use tokio::sync::oneshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerDispatchError {
  Closed,
  WorkerDropped,
  Cancelled,
}

impl Display for WorkerDispatchError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    formatter.write_str(match self {
      Self::Closed => "worker dispatcher is closed",
      Self::WorkerDropped => "worker dropped before returning the job result",
      Self::Cancelled => "worker job was cancelled by its dispatcher waiter",
    })
  }
}

impl Error for WorkerDispatchError {}

pub struct WorkerDispatchFailure<I> {
  error: WorkerDispatchError,
  input: Option<Box<I>>,
}

impl<I> WorkerDispatchFailure<I> {
  pub fn error(&self) -> WorkerDispatchError {
    self.error
  }

  pub fn into_parts(self) -> (WorkerDispatchError, Option<Box<I>>) {
    (self.error, self.input)
  }
}

pub struct WorkerJob<I, O> {
  id: u64,
  enqueued_at: Instant,
  input: Option<Box<I>>,
  result_tx: Option<oneshot::Sender<Result<Box<O>, WorkerDispatchFailure<I>>>>,
}

impl<I, O> WorkerJob<I, O> {
  pub fn id(&self) -> u64 {
    self.id
  }

  pub fn queue_duration(&self) -> Duration {
    self.enqueued_at.elapsed()
  }

  pub fn input(&self) -> &I {
    self.input.as_deref().expect("worker job input was taken")
  }

  pub fn input_mut(&mut self) -> &mut I {
    self
      .input
      .as_deref_mut()
      .expect("worker job input was taken")
  }

  pub fn take_input(&mut self) -> Box<I> {
    self.input.take().expect("worker job input was taken")
  }

  pub fn is_cancelled(&self) -> bool {
    self
      .result_tx
      .as_ref()
      .is_none_or(oneshot::Sender::is_closed)
  }

  pub fn complete(mut self, output: Box<O>) {
    if let Some(result_tx) = self.result_tx.take() {
      let _ = result_tx.send(Ok(output));
    }
  }

  pub fn fail(mut self, error: WorkerDispatchError) {
    if let Some(result_tx) = self.result_tx.take() {
      let _ = result_tx.send(Err(WorkerDispatchFailure {
        error,
        input: self.input.take(),
      }));
    }
  }
}

impl<I, O> Drop for WorkerJob<I, O> {
  fn drop(&mut self) {
    if let Some(result_tx) = self.result_tx.take() {
      let _ = result_tx.send(Err(WorkerDispatchFailure {
        error: WorkerDispatchError::WorkerDropped,
        input: self.input.take(),
      }));
    }
  }
}

struct DispatcherInner<I, O> {
  next_job_id: AtomicU64,
  sender: Sender<Box<WorkerJob<I, O>>>,
  receiver: Receiver<Box<WorkerJob<I, O>>>,
}

pub struct WorkerDispatcher<I, O> {
  inner: Arc<DispatcherInner<I, O>>,
}

impl<I, O> Clone for WorkerDispatcher<I, O> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<I, O> WorkerDispatcher<I, O> {
  pub fn unbounded() -> Self {
    let (sender, receiver) = async_channel::unbounded();
    Self {
      inner: Arc::new(DispatcherInner {
        next_job_id: AtomicU64::new(1),
        sender,
        receiver,
      }),
    }
  }

  pub async fn dispatch(&self, input: Box<I>) -> Result<Box<O>, WorkerDispatchFailure<I>> {
    let (result_tx, result_rx) = oneshot::channel();
    let job = Box::new(WorkerJob {
      id: self.inner.next_job_id.fetch_add(1, Ordering::Relaxed),
      enqueued_at: Instant::now(),
      input: Some(input),
      result_tx: Some(result_tx),
    });

    if let Err(error) = self.inner.sender.send(job).await {
      let mut job = error.into_inner();
      return Err(WorkerDispatchFailure {
        error: WorkerDispatchError::Closed,
        input: job.input.take(),
      });
    }

    result_rx.await.unwrap_or(Err(WorkerDispatchFailure {
      error: WorkerDispatchError::WorkerDropped,
      input: None,
    }))
  }

  pub async fn recv(&self) -> Option<Box<WorkerJob<I, O>>> {
    self.inner.receiver.recv().await.ok()
  }

  pub fn close(&self) {
    self.inner.sender.close();
    while let Ok(job) = self.inner.receiver.try_recv() {
      job.fail(WorkerDispatchError::Closed);
    }
  }
}
