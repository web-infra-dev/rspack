use std::{
  collections::HashSet,
  sync::{mpsc::Receiver as StdReceiver, Arc},
};

use tokio::sync::Mutex;

use super::EventHandler;
use crate::watcher::FsEvent;

type SafetyFiles = SafetyCollection<HashSet<String>>;
type SafetyCollection<T> = Arc<Mutex<T>>;
type SafetyReceiver<T> = Arc<Mutex<StdReceiver<T>>>;

pub struct WatcherExecutor {
  aggregate_timeout: u32,
  rx: SafetyReceiver<FsEvent>,
  changed_files: SafetyFiles,
  deleted_files: SafetyFiles,
}

const DEFAULT_AGGREGATE_TIMEOUT: u32 = 500; // Default timeout in milliseconds

enum ExecEvent {
  Execute,
  Close,
}

impl WatcherExecutor {
  pub fn new(rx: StdReceiver<FsEvent>, aggregate_timeout: Option<u32>) -> Self {
    Self {
      rx: Arc::new(Mutex::new(rx)),
      changed_files: Default::default(),
      deleted_files: Default::default(),
      aggregate_timeout: aggregate_timeout.unwrap_or(DEFAULT_AGGREGATE_TIMEOUT),
    }
  }

  pub async fn execute(&self, event_handler: Box<dyn EventHandler + Send + Sync>) {
    let (exec_tx, exec_rx) = std::sync::mpsc::channel::<ExecEvent>();

    let changed_files = Arc::clone(&self.changed_files);
    let deleted_files = Arc::clone(&self.deleted_files);
    let rx = Arc::clone(&self.rx);

    self.execute_aggregate_handler(exec_rx, event_handler);

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

    // TODO: Handle the error properly
    // Spawn the future to run in the Tokio runtime
    tokio::spawn(future).await.unwrap();
  }

  fn execute_aggregate_handler(
    &self,
    rx: StdReceiver<ExecEvent>,
    event_handler: Box<dyn EventHandler + Send + Sync>,
  ) {
    let aggregate_timeout = self.aggregate_timeout as u64;
    let changed_files = Arc::clone(&self.changed_files);
    let deleted_files = Arc::clone(&self.deleted_files);

    let future = async move {
      while let Ok(event) = rx.recv() {
        match event {
          ExecEvent::Execute => {
            // tokio::time::sleep(tokio::time::Duration::from_millis(aggregate_timeout)).await;
            let mut changed_files = changed_files.lock().await;
            let mut deleted_files = deleted_files.lock().await;
            if changed_files.is_empty() && deleted_files.is_empty() {
              continue; // No files to process, skip execution
            }

            let chaned_files = std::mem::take(&mut *changed_files);
            let deleted_files = std::mem::take(&mut *deleted_files);
            event_handler
              .on_event_handle(chaned_files, deleted_files)
              .await;

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
