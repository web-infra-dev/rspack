use std::{
  any::Any,
  fmt,
  future::Future,
  panic::AssertUnwindSafe,
  pin::Pin,
  sync::Mutex,
  task::{Context, Poll},
  time::SystemTime,
};

use futures::FutureExt;
use rspack_error::error;
use rspack_paths::ArcPath;
use tokio::sync::{mpsc, watch};

use crate::{
  EventAggregateHandler, EventHandler, FsEventKind, FsWatcher, FsWatcherIgnored, FsWatcherOptions,
};

type WatchPaths = (Vec<ArcPath>, Vec<ArcPath>);
type HandleResult<T> = std::result::Result<T, FsWatcherHandleError>;

enum FsWatcherCommand {
  Watch {
    files: WatchPaths,
    directories: WatchPaths,
    missing: WatchPaths,
    start_time: SystemTime,
    event_handler: Box<dyn EventAggregateHandler + Send>,
    event_handler_undelayed: Box<dyn EventHandler + Send>,
  },
  TriggerEvent {
    path: ArcPath,
    kind: FsEventKind,
  },
  Pause,
  Close,
}

type TaskOutput = (HandleResult<()>, mpsc::UnboundedReceiver<FsWatcherCommand>);

#[derive(Clone)]
enum FsWatcherCloseState {
  Open,
  Closing,
  Closed(HandleResult<()>),
}

#[derive(Debug, Clone)]
pub enum FsWatcherHandleError {
  Closed(FsWatcherOperation),
  Unavailable,
  Internal(rspack_error::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsWatcherOperation {
  Watch,
  TriggerEvent,
  Pause,
}

impl fmt::Display for FsWatcherHandleError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Closed(operation) => write!(
        formatter,
        "The file watcher cannot {operation} after it has been closed."
      ),
      Self::Unavailable => formatter.write_str("The file watcher is unavailable."),
      Self::Internal(error) => error.fmt(formatter),
    }
  }
}

impl fmt::Display for FsWatcherOperation {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Watch => formatter.write_str("watch"),
      Self::TriggerEvent => formatter.write_str("trigger events"),
      Self::Pause => formatter.write_str("pause"),
    }
  }
}

impl std::error::Error for FsWatcherHandleError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::Internal(error) => Some(error),
      _ => None,
    }
  }
}

/// Drives the exclusively owned file watcher and publishes a terminal result when dropped.
pub struct FsWatcherTask {
  future: Option<Pin<Box<dyn Future<Output = TaskOutput> + Send + 'static>>>,
  close_state_tx: watch::Sender<FsWatcherCloseState>,
  completed: bool,
}

impl FsWatcherTask {
  fn new(
    future: Pin<Box<dyn Future<Output = TaskOutput> + Send + 'static>>,
    close_state_tx: watch::Sender<FsWatcherCloseState>,
  ) -> Self {
    Self {
      future: Some(future),
      close_state_tx,
      completed: false,
    }
  }
}

impl Future for FsWatcherTask {
  type Output = ();

  fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
    let (result, command_rx) = match self.future.as_mut() {
      Some(future) => match future.as_mut().poll(context) {
        Poll::Ready(output) => output,
        Poll::Pending => return Poll::Pending,
      },
      None => return Poll::Ready(()),
    };

    self.completed = true;
    publish_close_result(&self.close_state_tx, result);
    self.future.take();
    drop(command_rx);

    Poll::Ready(())
  }
}

impl Drop for FsWatcherTask {
  fn drop(&mut self) {
    if !self.completed {
      self.future.take();
      self.completed = true;
      publish_close_result(&self.close_state_tx, Err(FsWatcherHandleError::Unavailable));
    }
  }
}

/// Provides concurrent access while a background task exclusively owns the file watcher.
pub struct FsWatcherHandle {
  command_tx: mpsc::UnboundedSender<FsWatcherCommand>,
  close_state_tx: watch::Sender<FsWatcherCloseState>,
  admission: Mutex<()>,
}

impl FsWatcherHandle {
  /// Creates a handle and a background task that the caller must run to drive it.
  pub fn new(options: FsWatcherOptions, ignored: FsWatcherIgnored) -> (Self, FsWatcherTask) {
    let watcher = FsWatcher::new(options, ignored);
    let (command_tx, mut command_rx) = mpsc::unbounded_channel();
    let (close_state_tx, _) = watch::channel(FsWatcherCloseState::Open);

    let task = FsWatcherTask::new(
      Box::pin(async move {
        let result = AssertUnwindSafe(run_fs_watcher_actor(watcher, &mut command_rx))
          .catch_unwind()
          .await;
        let result = match result {
          Ok(result) => result,
          Err(payload) => Err(FsWatcherHandleError::Internal(error!(panic_message(
            payload
          )))),
        };

        // The receiver stays alive until FsWatcherTask publishes the exact terminal result.
        (result, command_rx)
      }),
      close_state_tx.clone(),
    );

    (
      Self {
        command_tx,
        close_state_tx,
        admission: Mutex::new(()),
      },
      task,
    )
  }

  #[allow(clippy::too_many_arguments)]
  pub fn watch(
    &self,
    files: (Vec<ArcPath>, Vec<ArcPath>),
    directories: (Vec<ArcPath>, Vec<ArcPath>),
    missing: (Vec<ArcPath>, Vec<ArcPath>),
    start_time: SystemTime,
    event_handler: Box<dyn EventAggregateHandler + Send>,
    event_handler_undelayed: Box<dyn EventHandler + Send>,
  ) -> std::result::Result<(), FsWatcherHandleError> {
    self.enqueue(
      FsWatcherCommand::Watch {
        files,
        directories,
        missing,
        start_time,
        event_handler,
        event_handler_undelayed,
      },
      FsWatcherOperation::Watch,
    )
  }

  pub fn trigger_event(
    &self,
    path: &ArcPath,
    kind: FsEventKind,
  ) -> std::result::Result<(), FsWatcherHandleError> {
    self.enqueue(
      FsWatcherCommand::TriggerEvent {
        path: path.clone(),
        kind,
      },
      FsWatcherOperation::TriggerEvent,
    )
  }

  pub fn pause(&self) -> std::result::Result<(), FsWatcherHandleError> {
    self.enqueue(FsWatcherCommand::Pause, FsWatcherOperation::Pause)
  }

  pub fn close(
    &self,
  ) -> impl Future<Output = std::result::Result<(), FsWatcherHandleError>> + Send + 'static {
    let close_state_rx = self.start_close();

    async move {
      let close_state_rx = close_state_rx?;
      wait_for_close(close_state_rx).await
    }
  }

  fn enqueue(&self, command: FsWatcherCommand, operation: FsWatcherOperation) -> HandleResult<()> {
    let _admission = self
      .admission
      .lock()
      .map_err(|_| FsWatcherHandleError::Unavailable)?;
    if !is_open(&self.close_state_tx) {
      return Err(FsWatcherHandleError::Closed(operation));
    }

    self
      .command_tx
      .send(command)
      .map_err(|_| FsWatcherHandleError::Unavailable)
  }

  fn start_close(&self) -> HandleResult<watch::Receiver<FsWatcherCloseState>> {
    let _admission = self
      .admission
      .lock()
      .map_err(|_| FsWatcherHandleError::Unavailable)?;
    let close_state_rx = self.close_state_tx.subscribe();
    let should_send_close = self.close_state_tx.send_if_modified(|state| {
      if matches!(state, FsWatcherCloseState::Open) {
        *state = FsWatcherCloseState::Closing;
        true
      } else {
        false
      }
    });

    if should_send_close {
      let _ = self.command_tx.send(FsWatcherCommand::Close);
    }

    Ok(close_state_rx)
  }
}

async fn run_fs_watcher_actor(
  mut watcher: FsWatcher,
  command_rx: &mut mpsc::UnboundedReceiver<FsWatcherCommand>,
) -> HandleResult<()> {
  while let Some(command) = command_rx.recv().await {
    match command {
      FsWatcherCommand::Watch {
        files,
        directories,
        missing,
        start_time,
        event_handler,
        event_handler_undelayed,
      } => {
        watcher
          .watch(
            into_iterators(files),
            into_iterators(directories),
            into_iterators(missing),
            start_time,
            event_handler,
            event_handler_undelayed,
          )
          .await;
      }
      FsWatcherCommand::TriggerEvent { path, kind } => watcher.trigger_event(&path, kind),
      FsWatcherCommand::Pause => watcher.pause().map_err(FsWatcherHandleError::Internal)?,
      FsWatcherCommand::Close => {
        return watcher
          .close()
          .await
          .map_err(FsWatcherHandleError::Internal);
      }
    }
  }

  watcher
    .close()
    .await
    .map_err(FsWatcherHandleError::Internal)
}

fn into_iterators(
  paths: WatchPaths,
) -> (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>) {
  (paths.0.into_iter(), paths.1.into_iter())
}

fn is_open(close_state_tx: &watch::Sender<FsWatcherCloseState>) -> bool {
  matches!(*close_state_tx.borrow(), FsWatcherCloseState::Open)
}

fn publish_close_result(
  close_state_tx: &watch::Sender<FsWatcherCloseState>,
  result: HandleResult<()>,
) {
  close_state_tx.send_if_modified(|state| {
    if matches!(state, FsWatcherCloseState::Closed(_)) {
      return false;
    }

    *state = FsWatcherCloseState::Closed(result);
    true
  });
}

async fn wait_for_close(
  mut close_state_rx: watch::Receiver<FsWatcherCloseState>,
) -> HandleResult<()> {
  loop {
    let result = {
      let state = close_state_rx.borrow_and_update();
      match &*state {
        FsWatcherCloseState::Closed(result) => Some(result.clone()),
        FsWatcherCloseState::Open | FsWatcherCloseState::Closing => None,
      }
    };

    if let Some(result) = result {
      return result;
    }

    close_state_rx
      .changed()
      .await
      .map_err(|_| FsWatcherHandleError::Unavailable)?;
  }
}

fn panic_message(payload: Box<dyn Any + Send + 'static>) -> String {
  if let Some(message) = payload.downcast_ref::<&str>() {
    (*message).to_string()
  } else if let Some(message) = payload.downcast_ref::<String>() {
    message.clone()
  } else {
    "Panic in file watcher task".to_string()
  }
}
