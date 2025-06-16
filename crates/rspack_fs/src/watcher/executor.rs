use std::{collections::HashSet, sync::Arc};

use tokio::sync::Mutex;

use super::{EventHandler, StdReceiver};
use crate::watcher::{FsEvent, StdSender};

type SafetyFiles = Safety<HashSet<String>>;
type Safety<T> = Arc<Mutex<T>>;
type SafetyReceiver<T> = Arc<Mutex<StdReceiver<T>>>;

/// `WatcherExecutor` is responsible for managing the execution of file system event handlers,
/// aggregating file change and delete events, and invoking the provided event handler after
/// a configurable aggregate timeout. It receives events from a channel, tracks changed and
/// deleted files, and coordinates the event handling logic.
pub struct Executor {
  aggregate_timeout: u32,
  rx: SafetyReceiver<FsEvent>,
  changed_files: SafetyFiles,
  deleted_files: SafetyFiles,
  exec_tx: StdSender<ExecEvent>,
  exec_rx: Safety<StdReceiver<ExecEvent>>,
  join_handle: Option<tokio::task::JoinHandle<()>>,
}

const DEFAULT_AGGREGATE_TIMEOUT: u32 = 50; // Default timeout in milliseconds

/// `ExecEvent` represents control events for the watcher executor loop.
/// - `Execute`: Indicates that an event (change or delete) has occurred and the handler should be triggered.
/// - `Close`: Indicates that the event receiver has been closed and the executor should stop.
enum ExecEvent {
  /// Trigger the execution of the event handler (e.g., after a file change or delete).
  Execute,
  /// Signal to close the executor loop (e.g., when the receiver is closed).
  Close,
}

impl Executor {
  /// Create a new `WatcherExecutor` with the given receiver and optional aggregate timeout.
  pub fn new(rx: StdReceiver<FsEvent>, aggregate_timeout: Option<u32>) -> Self {
    let (exec_tx, exec_rx) = std::sync::mpsc::channel::<ExecEvent>();

    Self {
      rx: Arc::new(Mutex::new(rx)),
      changed_files: Default::default(),
      deleted_files: Default::default(),
      exec_tx,
      join_handle: None,
      exec_rx: Arc::new(Mutex::new(exec_rx)),
      aggregate_timeout: aggregate_timeout.unwrap_or(DEFAULT_AGGREGATE_TIMEOUT),
    }
  }

  /// Execute the watcher executor loop.
  /// TODO: handle missing files
  pub fn wait_for_execute(&mut self, event_handler: Box<dyn EventHandler + Send + Sync>) {
    if self.join_handle.is_none() {
      let changed_files = Arc::clone(&self.changed_files);
      let deleted_files = Arc::clone(&self.deleted_files);
      let rx = Arc::clone(&self.rx);
      let exec_tx = self.exec_tx.clone();
      // Use an async block to handle the events
      let future = async move {
        loop {
          match rx.lock().await.recv() {
            Ok(event) => match event.kind {
              super::FsEventKind::Change => {
                let path = event.path.to_string_lossy().to_string();
                changed_files.lock().await.insert(path);
                exec_tx.send(ExecEvent::Execute).unwrap_or_default();
              }
              super::FsEventKind::Delete => {
                let path = event.path.to_string_lossy().to_string();
                deleted_files.lock().await.insert(path);
                exec_tx.send(ExecEvent::Execute).unwrap_or_default();
              }
            },
            Err(_) => {
              exec_tx.send(ExecEvent::Close).unwrap_or_default();
              // Handle the case where the receiver is closed
              // Receiver closed, stopping watcher.
              break;
            }
          }
        }
      };
      self.join_handle = Some(tokio::spawn(future));
    }

    self.execute_aggregate_handler(event_handler);
  }

  /// Executes the aggregate handler for file changes and deletions.
  /// It waits for a configurable aggregate timeout before executing the handler.
  fn execute_aggregate_handler(&self, event_handler: Box<dyn EventHandler + Send + Sync>) {
    let aggregate_timeout = self.aggregate_timeout as u64;
    let changed_files = Arc::clone(&self.changed_files);
    let deleted_files = Arc::clone(&self.deleted_files);
    let rx = Arc::clone(&self.exec_rx);

    let future = async move {
      while let Ok(event) = rx.lock().await.recv() {
        match event {
          ExecEvent::Execute => {
            tokio::time::sleep(tokio::time::Duration::from_millis(aggregate_timeout)).await;
            let mut changed_files = changed_files.lock().await;
            let mut deleted_files = deleted_files.lock().await;
            if changed_files.is_empty() && deleted_files.is_empty() {
              continue; // No files to process, skip execution
            }

            let changed_files = std::mem::take(&mut *changed_files);
            let deleted_files = std::mem::take(&mut *deleted_files);
            event_handler
              .on_event_handle(changed_files, deleted_files)
              .await;
            break;

            // Handle the execution of the aggregate logic
          }
          ExecEvent::Close => {
            // Handle the close event if needed
            break;
          }
        }
      }
    };

    tokio::spawn(future);
  }
}
