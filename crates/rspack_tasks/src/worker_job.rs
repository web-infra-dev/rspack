use std::{
  error::Error,
  fmt::{self, Display, Formatter},
};

use async_channel::Sender;
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
      Self::Closed => "worker queue is closed",
      Self::WorkerDropped => "worker dropped before returning the job result",
      Self::Cancelled => "worker job was cancelled by its dispatch waiter",
    })
  }
}

impl Error for WorkerDispatchError {}

pub enum WorkerFailure<E> {
  Dispatch(WorkerDispatchError),
  Task(E),
}

impl<E: Display> Display for WorkerFailure<E> {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Dispatch(error) => error.fmt(formatter),
      Self::Task(error) => error.fmt(formatter),
    }
  }
}

pub struct WorkerJobFailure<T, E> {
  error: WorkerFailure<E>,
  input: Option<Box<T>>,
}

impl<T, E> WorkerJobFailure<T, E> {
  pub fn into_parts(self) -> (WorkerFailure<E>, Option<Box<T>>) {
    (self.error, self.input)
  }
}

pub struct WorkerJob<T, E> {
  input: Option<Box<T>>,
  result_tx: Option<oneshot::Sender<Result<Box<T>, WorkerJobFailure<T, E>>>>,
}

impl<T, E> WorkerJob<T, E> {
  pub async fn dispatch(
    sender: &Sender<Box<Self>>,
    input: Box<T>,
  ) -> Result<Box<T>, WorkerJobFailure<T, E>> {
    let (result_tx, result_rx) = oneshot::channel();
    let job = Box::new(Self {
      input: Some(input),
      result_tx: Some(result_tx),
    });

    if let Err(error) = sender.send(job).await {
      let mut job = error.into_inner();
      return Err(WorkerJobFailure {
        error: WorkerFailure::Dispatch(WorkerDispatchError::Closed),
        input: job.input.take(),
      });
    }

    result_rx.await.unwrap_or(Err(WorkerJobFailure {
      error: WorkerFailure::Dispatch(WorkerDispatchError::WorkerDropped),
      input: None,
    }))
  }

  pub fn input(&self) -> &T {
    self.input.as_deref().expect("worker job input was taken")
  }

  pub fn input_mut(&mut self) -> &mut T {
    self
      .input
      .as_deref_mut()
      .expect("worker job input was taken")
  }

  pub fn is_cancelled(&self) -> bool {
    self
      .result_tx
      .as_ref()
      .is_none_or(oneshot::Sender::is_closed)
  }

  pub fn complete(mut self) {
    let output = self.input.take().expect("worker job input was taken");
    if let Some(result_tx) = self.result_tx.take() {
      let _ = result_tx.send(Ok(output));
    }
  }

  pub fn fail(mut self, error: E) {
    if let Some(result_tx) = self.result_tx.take() {
      let _ = result_tx.send(Err(WorkerJobFailure {
        error: WorkerFailure::Task(error),
        input: self.input.take(),
      }));
    }
  }

  pub fn fail_dispatch(mut self, error: WorkerDispatchError) {
    if let Some(result_tx) = self.result_tx.take() {
      let _ = result_tx.send(Err(WorkerJobFailure {
        error: WorkerFailure::Dispatch(error),
        input: self.input.take(),
      }));
    }
  }
}

impl<T, E> Drop for WorkerJob<T, E> {
  fn drop(&mut self) {
    if let Some(result_tx) = self.result_tx.take() {
      let _ = result_tx.send(Err(WorkerJobFailure {
        error: WorkerFailure::Dispatch(WorkerDispatchError::WorkerDropped),
        input: self.input.take(),
      }));
    }
  }
}
