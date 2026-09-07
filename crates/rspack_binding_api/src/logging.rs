use std::{
  sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
  },
  time::Duration,
};

use napi::Either;
use napi_derive::napi;
use rspack_core::{InfrastructureLogEvent, InfrastructureLogSink, LogType};
use tokio::{
  sync::{mpsc, oneshot},
  time::sleep,
};

use crate::compiler_scoped_tsfn::CompilerScopedTsFnHandle;

const INFRASTRUCTURE_LOG_BATCH_INTERVAL: Duration = Duration::from_millis(100);
const INFRASTRUCTURE_LOG_BATCH_SIZE: usize = 8;
const INFRASTRUCTURE_LOG_QUEUE_CAPACITY: usize = 32;

type InfrastructureLogCallback = CompilerScopedTsFnHandle<Vec<JsLog>, ()>;
type ShutdownRequest = Option<oneshot::Sender<()>>;

#[napi(object, object_from_js = false)]
pub struct JsLog {
  pub name: String,
  pub r#type: String,
  pub args: Vec<Either<String, f64>>,
  pub trace: Option<Vec<String>>,
}

impl From<(Arc<str>, LogType)> for JsLog {
  fn from((name, log_type): (Arc<str>, LogType)) -> Self {
    let name = name.to_string();
    let (r#type, args, trace) = match log_type {
      LogType::Error { message, trace } => ("error", vec![Either::A(message)], Some(trace)),
      LogType::Warn { message, trace } => ("warn", vec![Either::A(message)], Some(trace)),
      LogType::Info { message } => ("info", vec![Either::A(message)], None),
      LogType::Log { message } => ("log", vec![Either::A(message)], None),
      LogType::Debug { message } => ("debug", vec![Either::A(message)], None),
      LogType::Trace { message, trace } => ("trace", vec![Either::A(message)], Some(trace)),
      LogType::Group { message } => ("group", vec![Either::A(message)], None),
      LogType::GroupCollapsed { message } => ("groupCollapsed", vec![Either::A(message)], None),
      LogType::GroupEnd => ("groupEnd", vec![], None),
      LogType::Profile { label } => ("profile", vec![Either::A(label.to_string())], None),
      LogType::ProfileEnd { label } => ("profileEnd", vec![Either::A(label.to_string())], None),
      LogType::Time {
        label,
        secs,
        subsec_nanos,
      } => (
        "time",
        vec![
          Either::A(label.to_string()),
          Either::B(secs as f64),
          Either::B(subsec_nanos as f64),
        ],
        None,
      ),
      LogType::Clear => ("clear", vec![], None),
      LogType::Status { message } => ("status", vec![Either::A(message)], None),
      LogType::Cache { label, hit, total } => (
        "cache",
        vec![Either::A(format!(
          "{label}: {:.1}% ({hit}/{total})",
          if total == 0 {
            0.0
          } else {
            hit as f32 / total as f32 * 100.0
          }
        ))],
        None,
      ),
    };
    Self {
      name,
      r#type: r#type.to_string(),
      args,
      trace,
    }
  }
}

impl From<InfrastructureLogEvent> for JsLog {
  fn from(event: InfrastructureLogEvent) -> Self {
    let mut log = Self::from((event.name, event.log_type));
    log.trace = None;
    log
  }
}

struct BatchedInfrastructureLogSink {
  sender: mpsc::Sender<InfrastructureLogEvent>,
  closed: Arc<AtomicBool>,
  dropped: Arc<AtomicUsize>,
}

impl InfrastructureLogSink for BatchedInfrastructureLogSink {
  fn emit(&self, event: InfrastructureLogEvent) {
    if self.closed.load(Ordering::Acquire) {
      return;
    }
    match self.sender.try_send(event) {
      Ok(()) => {}
      Err(mpsc::error::TrySendError::Full(event)) => {
        self.dropped.fetch_add(1, Ordering::Relaxed);
        log_critical_fallback(event);
      }
      Err(mpsc::error::TrySendError::Closed(event)) => {
        log_critical_fallback(event);
      }
    }
  }
}

fn log_critical_fallback(event: InfrastructureLogEvent) {
  match event.log_type {
    LogType::Error { message, .. } => {
      tracing::error!(name = %event.name, "Infrastructure logging queue is unavailable: {message}")
    }
    LogType::Warn { message, .. } => {
      tracing::warn!(name = %event.name, "Infrastructure logging queue is unavailable: {message}")
    }
    _ => {}
  }
}

pub struct InfrastructureLogDispatcher {
  shutdown_sender: mpsc::UnboundedSender<ShutdownRequest>,
  closed: Arc<AtomicBool>,
}

impl InfrastructureLogDispatcher {
  pub fn new(callback: InfrastructureLogCallback) -> (Arc<dyn InfrastructureLogSink>, Arc<Self>) {
    let (sender, receiver) = mpsc::channel(INFRASTRUCTURE_LOG_QUEUE_CAPACITY);
    let (shutdown_sender, shutdown_receiver) = mpsc::unbounded_channel();
    let closed = Arc::new(AtomicBool::new(false));
    let dropped = Arc::new(AtomicUsize::new(0));
    let sink = Arc::new(BatchedInfrastructureLogSink {
      sender,
      closed: closed.clone(),
      dropped: dropped.clone(),
    });
    rspack_napi::runtime::spawn(run_dispatcher(
      receiver,
      shutdown_receiver,
      callback,
      dropped,
    ));
    (
      sink,
      Arc::new(Self {
        shutdown_sender,
        closed,
      }),
    )
  }

  pub async fn shutdown(&self) {
    if self.closed.swap(true, Ordering::AcqRel) {
      return;
    }
    let (sender, receiver) = oneshot::channel();
    if self.shutdown_sender.send(Some(sender)).is_ok() {
      let _ = receiver.await;
    }
  }
}

impl Drop for InfrastructureLogDispatcher {
  fn drop(&mut self) {
    if !self.closed.swap(true, Ordering::AcqRel) {
      let _ = self.shutdown_sender.send(None);
    }
  }
}

async fn run_dispatcher(
  mut receiver: mpsc::Receiver<InfrastructureLogEvent>,
  mut shutdown_receiver: mpsc::UnboundedReceiver<ShutdownRequest>,
  callback: InfrastructureLogCallback,
  dropped: Arc<AtomicUsize>,
) {
  loop {
    let first = tokio::select! {
      biased;
      shutdown = shutdown_receiver.recv() => {
        flush_and_shutdown(&mut receiver, &callback, &dropped, shutdown.flatten()).await;
        return;
      }
      event = receiver.recv() => {
        let Some(event) = event else {
          return;
        };
        event
      }
    };

    let mut batch = Vec::with_capacity(INFRASTRUCTURE_LOG_BATCH_SIZE);
    batch.push(first);
    let delay = sleep(INFRASTRUCTURE_LOG_BATCH_INTERVAL);
    tokio::pin!(delay);
    let mut shutdown = None;

    while batch.len() < INFRASTRUCTURE_LOG_BATCH_SIZE {
      tokio::select! {
        biased;
        request = shutdown_receiver.recv() => {
          shutdown = Some(request.flatten());
          break;
        }
        event = receiver.recv() => {
          let Some(event) = event else {
            break;
          };
          batch.push(event);
        }
        _ = &mut delay => break,
      }
    }

    dispatch_batch(&callback, &dropped, batch).await;
    if let Some(shutdown) = shutdown {
      flush_and_shutdown(&mut receiver, &callback, &dropped, shutdown).await;
      return;
    }
  }
}

async fn flush_and_shutdown(
  receiver: &mut mpsc::Receiver<InfrastructureLogEvent>,
  callback: &InfrastructureLogCallback,
  dropped: &AtomicUsize,
  shutdown: ShutdownRequest,
) {
  // Prevent a producer that raced with `closed` from enqueueing after the final drain.
  receiver.close();
  let mut batch = Vec::with_capacity(INFRASTRUCTURE_LOG_BATCH_SIZE);
  while let Some(event) = receiver.recv().await {
    batch.push(event);
    if batch.len() == INFRASTRUCTURE_LOG_BATCH_SIZE {
      dispatch_batch(callback, dropped, std::mem::take(&mut batch)).await;
    }
  }
  dispatch_batch(callback, dropped, batch).await;
  if let Some(shutdown) = shutdown {
    let _ = shutdown.send(());
  }
}

async fn dispatch_batch(
  callback: &InfrastructureLogCallback,
  dropped: &AtomicUsize,
  mut batch: Vec<InfrastructureLogEvent>,
) {
  let dropped = dropped.swap(0, Ordering::Relaxed);
  if dropped != 0 {
    batch.insert(
      0,
      InfrastructureLogEvent {
        name: Arc::from("rspack.infrastructure"),
        log_type: LogType::Warn {
          message: format!("Dropped {dropped} infrastructure log messages"),
          trace: vec![],
        },
      },
    );
  }
  if batch.is_empty() {
    return;
  }
  let logs = batch.into_iter().map(Into::into).collect::<Vec<_>>();
  if let Err(error) = callback.call_with_sync(logs).await {
    tracing::warn!("Sending infrastructure logs to JavaScript failed: {error}");
  }
}
