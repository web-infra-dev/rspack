use std::{
  collections::HashSet,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use tokio::sync::{
  mpsc::{self, UnboundedReceiver, UnboundedSender},
  Mutex,
};

use super::EventHandler;
use crate::watcher::FsEvent;

type ThreadSafetyReceiver<T> = ThreadSafety<UnboundedReceiver<T>>;
type ThreadSafety<T> = Arc<Mutex<T>>;

#[derive(Debug, Default)]
struct FilesData {
  changed: HashSet<String>,
  deleted: HashSet<String>,
}

impl FilesData {
  fn is_empty(&self) -> bool {
    self.changed.is_empty() && self.deleted.is_empty()
  }
}

/// `WatcherExecutor` is responsible for managing the execution of file system event handlers,
/// aggregating file change and delete events, and invoking the provided event handler after
/// a configurable aggregate timeout. It receives events from a channel, tracks changed and
/// deleted files, and coordinates the event handling logic.
pub struct Executor {
  aggregate_timeout: u32,
  rx: ThreadSafety<UnboundedReceiver<FsEvent>>,
  files_data: ThreadSafety<FilesData>,
  exec_aggreate_tx: UnboundedSender<ExecAggreateEvent>,
  exec_aggreate_rx: ThreadSafetyReceiver<ExecAggreateEvent>,
  exec_tx: UnboundedSender<ExecEvent>,
  exec_rx: ThreadSafetyReceiver<ExecEvent>,
  paused: Arc<AtomicBool>,
  aggreate_running: Arc<AtomicBool>,
  start_waiting: bool,
  execute_handler: Option<ExecuteHandler>,
  execute_aggregate_handler: Option<ExecuteAggregateHandler>,
}

const DEFAULT_AGGREGATE_TIMEOUT: u32 = 50; // Default timeout in milliseconds

/// `ExecEvent` represents control events for the watcher executor loop.
/// - `Execute`: Indicates that an event (change or delete) has occurred and the handler should be triggered.
/// - `Close`: Indicates that the event receiver has been closed and the executor should stop.
enum ExecAggreateEvent {
  /// Trigger the execution of the event handler (e.g., after a file change or delete).
  Execute,
  /// Signal to close the executor loop (e.g., when the receiver is closed).
  Close,
}

enum ExecEvent {
  Execute(FsEvent),
  Close,
}

impl Executor {
  /// Create a new `WatcherExecutor` with the given receiver and optional aggregate timeout.
  pub fn new(rx: UnboundedReceiver<FsEvent>, aggregate_timeout: Option<u32>) -> Self {
    let (exec_aggreate_tx, exec_aggreate_rx) = mpsc::unbounded_channel::<ExecAggreateEvent>();
    let (exec_tx, exec_rx) = mpsc::unbounded_channel::<ExecEvent>();

    Self {
      start_waiting: false,
      aggreate_running: Arc::new(AtomicBool::new(false)),
      paused: Arc::new(AtomicBool::new(false)),
      rx: Arc::new(Mutex::new(rx)),
      files_data: Default::default(),
      exec_aggreate_tx,
      exec_aggreate_rx: Arc::new(Mutex::new(exec_aggreate_rx)),
      exec_rx: Arc::new(Mutex::new(exec_rx)),
      exec_tx,
      execute_aggregate_handler: None,
      execute_handler: None,
      aggregate_timeout: aggregate_timeout.unwrap_or(DEFAULT_AGGREGATE_TIMEOUT),
    }
  }

  /// Pause the aggregate executor, it will not execute the event handler until resume.
  pub fn pause(&self) {
    self
      .paused
      .store(true, std::sync::atomic::Ordering::Relaxed);
  }

  /// Abort all executor.
  fn abort(&self) {
    if let Some(execute_aggregate_handler) = &self.execute_aggregate_handler {
      execute_aggregate_handler.abort();
    }
    if let Some(execute_handler) = &self.execute_handler {
      execute_handler.abort();
    }
  }

  /// Abort all executor and close the receiver.
  pub fn close(&self) {
    self.abort();
  }

  /// Execute the watcher executor loop.
  pub async fn wait_for_execute(&mut self, event_handler: Box<dyn EventHandler + Send + Sync>) {
    if !self.start_waiting {
      let files = Arc::clone(&self.files_data);

      let rx = Arc::clone(&self.rx);
      let exec_aggreate_tx = self.exec_aggreate_tx.clone();
      let exec_tx = self.exec_tx.clone();
      let paused = Arc::clone(&self.paused);
      let aggreate_running = Arc::clone(&self.aggreate_running);

      let future = async move {
        while let Some(event) = rx.lock().await.recv().await {
          let path = event.path.to_string_lossy().to_string();
          match event.kind {
            super::FsEventKind::Change => {
              files.lock().await.changed.insert(path);
            }
            super::FsEventKind::Remove => {
              files.lock().await.deleted.insert(path);
            }
            super::FsEventKind::Create => {
              files.lock().await.changed.insert(path);
            }
          };

          if !paused.load(Ordering::Relaxed) && !aggreate_running.load(Ordering::Relaxed) {
            let _ = exec_aggreate_tx.send(ExecAggreateEvent::Execute);
          }
          let _ = exec_tx.send(ExecEvent::Execute(event));
        }

        let _ = exec_aggreate_tx.send(ExecAggreateEvent::Close);
        let _ = exec_tx.send(ExecEvent::Close);
      };

      tokio::spawn(future);
      self.start_waiting = true;
    }

    self.paused.store(false, Ordering::Relaxed);
    // abort the previous handlers if they exist
    // sleep 1ms to make sure the previous handlers are aborted
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    self.abort();

    self.run_execute_handler(event_handler);
  }

  fn run_execute_handler(&mut self, event_handler: Box<dyn EventHandler + Send + Sync>) {
    let event_handler = Arc::new(event_handler);

    self.execute_aggregate_handler = Some(ExecuteAggregateHandler::new(
      Arc::clone(&event_handler),
      Arc::clone(&self.exec_aggreate_rx),
      Arc::clone(&self.files_data),
      self.aggregate_timeout as u64,
      Arc::clone(&self.aggreate_running),
    ));

    self.execute_handler = Some(ExecuteHandler::new(
      event_handler,
      Arc::clone(&self.exec_rx),
    ));
  }
}

struct ExecuteHandler {
  task: tokio::task::JoinHandle<()>,
}

impl ExecuteHandler {
  fn new(
    event_handler: Arc<Box<dyn EventHandler + Send + Sync>>,
    exec_rx: ThreadSafetyReceiver<ExecEvent>,
  ) -> Self {
    let future = async move {
      while let Some(event) = exec_rx.lock().await.recv().await {
        match event {
          ExecEvent::Execute(event) => {
            let path = event.path.to_string_lossy().to_string();
            match event.kind {
              super::FsEventKind::Change | super::FsEventKind::Create => {
                if event_handler.on_change(path).await.is_err() {
                  break;
                }
              }
              super::FsEventKind::Remove => {
                if event_handler.on_delete(path).await.is_err() {
                  break;
                }
              }
            }
          }
          ExecEvent::Close => {
            break;
          }
        }
      }
    };

    Self {
      task: tokio::spawn(future),
    }
  }

  fn abort(&self) {
    self.task.abort();
  }
}

struct ExecuteAggregateHandler {
  task: tokio::task::JoinHandle<()>,
}

impl ExecuteAggregateHandler {
  fn new(
    event_handler: Arc<Box<dyn EventHandler + Send + Sync>>,
    exec_aggreate_rx: ThreadSafetyReceiver<ExecAggreateEvent>,
    files: ThreadSafety<FilesData>,
    aggregate_timeout: u64,
    running: Arc<AtomicBool>,
  ) -> Self {
    let future = async move {
      let aggreate_rx = {
        // release the lock on exec_aggreate_rx
        // and wait for the next event
        let mut exec_aggreate_rx_guard = exec_aggreate_rx.lock().await;
        match exec_aggreate_rx_guard.recv().await {
          Some(event) => event,
          None => return,
        }
      };

      match aggreate_rx {
        ExecAggreateEvent::Execute => {
          running.store(true, Ordering::Relaxed);
          // Wait for the aggregate timeout before executing the handler
          tokio::time::sleep(tokio::time::Duration::from_millis(aggregate_timeout)).await;

          // Get the files to process
          let files = {
            let mut files = files.lock().await;
            if files.is_empty() {
              return;
            }
            std::mem::take(&mut *files)
          };

          // Call the event handler with the changed and deleted files
          let _ = event_handler
            .on_event_handle(files.changed, files.deleted)
            .await;
          running.store(false, Ordering::Relaxed);
        }
        _ => (),
      }
    };

    let task = tokio::spawn(future);

    Self { task }
  }

  fn abort(&self) {
    self.task.abort();
  }
}
