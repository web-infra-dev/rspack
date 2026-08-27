use std::{
  error::Error,
  fmt::{self, Display, Formatter},
  sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
  },
  time::{Duration, Instant},
};

use async_channel::{Receiver, Sender};
use tokio::sync::{Notify, oneshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerDispatchError {
  Closed,
  NoConsumers,
  ConsumerDropped,
  Cancelled,
}

impl Display for WorkerDispatchError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    formatter.write_str(match self {
      Self::Closed => "worker dispatcher is closed",
      Self::NoConsumers => "worker dispatcher has no registered consumers",
      Self::ConsumerDropped => "worker consumer dropped before returning the job result",
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
        error: WorkerDispatchError::ConsumerDropped,
        input: self.input.take(),
      }));
    }
  }
}

struct DispatcherInner<I, O> {
  next_job_id: AtomicU64,
  consumers: AtomicUsize,
  sender: Sender<Box<WorkerJob<I, O>>>,
  receiver: Receiver<Box<WorkerJob<I, O>>>,
  consumer_state_changed: Notify,
  // Serializes last-consumer draining with registration so a newly registered consumer cannot
  // race a stale "no consumers" decision. The queue itself remains a lock-free async MPMC.
  consumer_gate: Mutex<()>,
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
  pub fn bounded(capacity: usize) -> Self {
    assert!(capacity > 0, "worker dispatcher capacity must be non-zero");
    let (sender, receiver) = async_channel::bounded(capacity);
    Self {
      inner: Arc::new(DispatcherInner {
        next_job_id: AtomicU64::new(1),
        consumers: AtomicUsize::new(0),
        sender,
        receiver,
        consumer_state_changed: Notify::new(),
        consumer_gate: Mutex::new(()),
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

    if self.inner.consumers.load(Ordering::Acquire) == 0 {
      let _gate = self
        .inner
        .consumer_gate
        .lock()
        .unwrap_or_else(|error| error.into_inner());
      if self.inner.consumers.load(Ordering::Acquire) == 0 {
        while let Ok(job) = self.inner.receiver.try_recv() {
          job.fail(WorkerDispatchError::NoConsumers);
        }
      }
    }

    result_rx.await.unwrap_or(Err(WorkerDispatchFailure {
      error: WorkerDispatchError::ConsumerDropped,
      input: None,
    }))
  }

  pub fn register_consumer(&self) -> Result<WorkerConsumer<I, O>, WorkerDispatchError> {
    let _gate = self
      .inner
      .consumer_gate
      .lock()
      .unwrap_or_else(|error| error.into_inner());
    if self.inner.sender.is_closed() {
      return Err(WorkerDispatchError::Closed);
    }
    self.inner.consumers.fetch_add(1, Ordering::AcqRel);
    self.inner.consumer_state_changed.notify_waiters();
    Ok(WorkerConsumer {
      inner: self.inner.clone(),
      registered: Arc::new(AtomicBool::new(true)),
      cancelled: Arc::new(Notify::new()),
    })
  }

  pub async fn consumer_count(&self) -> usize {
    self.inner.consumers.load(Ordering::Acquire)
  }

  pub async fn wait_for_consumer(&self) -> Result<(), WorkerDispatchError> {
    loop {
      let changed = self.inner.consumer_state_changed.notified();
      tokio::pin!(changed);
      changed.as_mut().enable();
      if self.inner.sender.is_closed() {
        return Err(WorkerDispatchError::Closed);
      }
      if self.inner.consumers.load(Ordering::Acquire) > 0 {
        return Ok(());
      }
      changed.await;
    }
  }

  pub fn close(&self) {
    self.inner.sender.close();
    self.inner.consumer_state_changed.notify_waiters();
    while let Ok(job) = self.inner.receiver.try_recv() {
      job.fail(WorkerDispatchError::Closed);
    }
  }
}

pub struct WorkerConsumer<I, O> {
  inner: Arc<DispatcherInner<I, O>>,
  registered: Arc<AtomicBool>,
  cancelled: Arc<Notify>,
}

pub struct WorkerConsumerHandle<I, O> {
  inner: Arc<DispatcherInner<I, O>>,
  registered: Arc<AtomicBool>,
  cancelled: Arc<Notify>,
}

impl<I, O> WorkerConsumer<I, O> {
  pub async fn recv(&self) -> Option<Box<WorkerJob<I, O>>> {
    if !self.registered.load(Ordering::Acquire) {
      return None;
    }
    tokio::select! {
      job = self.inner.receiver.recv() => job.ok(),
      _ = self.cancelled.notified() => None,
    }
  }

  pub fn handle(&self) -> WorkerConsumerHandle<I, O> {
    WorkerConsumerHandle {
      inner: self.inner.clone(),
      registered: self.registered.clone(),
      cancelled: self.cancelled.clone(),
    }
  }

  pub fn unregister(&self) {
    unregister_consumer(&self.inner, &self.registered, &self.cancelled);
  }
}

impl<I, O> Drop for WorkerConsumer<I, O> {
  fn drop(&mut self) {
    self.unregister();
  }
}

impl<I, O> WorkerConsumerHandle<I, O> {
  pub fn unregister(&self) {
    unregister_consumer(&self.inner, &self.registered, &self.cancelled);
  }
}

fn unregister_consumer<I, O>(
  inner: &DispatcherInner<I, O>,
  registered: &AtomicBool,
  cancelled: &Notify,
) {
  if !registered.swap(false, Ordering::AcqRel) {
    return;
  }
  let _gate = inner
    .consumer_gate
    .lock()
    .unwrap_or_else(|error| error.into_inner());
  let previous = inner.consumers.fetch_sub(1, Ordering::AcqRel);
  debug_assert!(previous > 0, "worker consumer count underflow");
  if previous == 1 && !inner.sender.is_closed() {
    while let Ok(job) = inner.receiver.try_recv() {
      job.fail(WorkerDispatchError::NoConsumers);
    }
  }
  inner.consumer_state_changed.notify_waiters();
  cancelled.notify_one();
}
