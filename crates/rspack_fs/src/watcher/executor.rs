use std::{collections::HashSet, sync::Arc};

use tokio::sync::{
  mpsc::{self, UnboundedReceiver, UnboundedSender},
  Mutex,
};

use super::EventHandler;
use crate::watcher::FsEvent;

type ThreadSafetyFiles = ThreadSafety<HashSet<String>>;
type ThreadSafetyReceiver<T> = ThreadSafety<UnboundedReceiver<T>>;
type ThreadSafety<T> = Arc<Mutex<T>>;

/// `WatcherExecutor` is responsible for managing the execution of file system event handlers,
/// aggregating file change and delete events, and invoking the provided event handler after
/// a configurable aggregate timeout. It receives events from a channel, tracks changed and
/// deleted files, and coordinates the event handling logic.
pub struct Executor {
  aggregate_timeout: u32,
  rx: ThreadSafety<UnboundedReceiver<FsEvent>>,
  changed_files: ThreadSafetyFiles,
  deleted_files: ThreadSafetyFiles,
  exec_aggreate_tx: UnboundedSender<ExecAggreateEvent>,
  exec_aggreate_rx: ThreadSafetyReceiver<ExecAggreateEvent>,
  exec_tx: UnboundedSender<ExecEvent>,
  exec_rx: ThreadSafetyReceiver<ExecEvent>,
  run: bool,
  // join_handle: Option<tokio::task::JoinHandle<()>>,
  /// The executor is responsible for executing the event handler after a configurable timeout.
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
      run: false,
      rx: Arc::new(Mutex::new(rx)),
      changed_files: Default::default(),
      deleted_files: Default::default(),
      exec_aggreate_tx,
      exec_aggreate_rx: Arc::new(Mutex::new(exec_aggreate_rx)),
      exec_rx: Arc::new(Mutex::new(exec_rx)),
      exec_tx,
      execute_aggregate_handler: None,
      execute_handler: None,
      // join_handle: None,
      aggregate_timeout: aggregate_timeout.unwrap_or(DEFAULT_AGGREGATE_TIMEOUT),
    }
  }

  /// stop the last handlers if they exist.
  pub async fn pause(&self) {
    if let Some(execute_aggregate_handler) = &self.execute_aggregate_handler {
      execute_aggregate_handler.stop().await;
    }
    if let Some(execute_handler) = &self.execute_handler {
      execute_handler.stop().await;
    }
  }

  /// Execute the watcher executor loop.
  pub async fn wait_for_execute(&mut self, event_handler: Box<dyn EventHandler + Send + Sync>) {
    if !self.run {
      let changed_files = Arc::clone(&self.changed_files);
      let deleted_files = Arc::clone(&self.deleted_files);

      let rx = Arc::clone(&self.rx);
      let exec_aggreate_tx = self.exec_aggreate_tx.clone();
      let exec_tx = self.exec_tx.clone();

      let future = async move {
        while let Some(event) = rx.lock().await.recv().await {
          let path = event.path.to_string_lossy().to_string();
          let files = match event.kind {
            super::FsEventKind::Change => &changed_files,
            super::FsEventKind::Remove => &deleted_files,
            super::FsEventKind::Create => &changed_files,
          };

          files.lock().await.insert(path);
          let _ = exec_aggreate_tx.send(ExecAggreateEvent::Execute);
          let _ = exec_tx.send(ExecEvent::Execute(event));
        }

        let _ = exec_aggreate_tx.send(ExecAggreateEvent::Close);
        let _ = exec_tx.send(ExecEvent::Close);
      };

      tokio::spawn(future);
      self.run = true;
    }
    self.pause().await;

    self.run_execute_handler(event_handler);
  }

  fn run_execute_handler(&mut self, event_handler: Box<dyn EventHandler + Send + Sync>) {
    let event_handler = Arc::new(event_handler);

    self.execute_aggregate_handler = Some(
      ExecuteAggregateHandler::new(
        Arc::clone(&event_handler),
        Arc::clone(&self.exec_aggreate_rx),
        Arc::clone(&self.changed_files),
        Arc::clone(&self.deleted_files),
        self.aggregate_timeout as u64,
      )
      .run(),
    );

    self.execute_handler =
      Some(ExecuteHandler::new(event_handler, Arc::clone(&self.exec_rx)).run());
  }
}

struct ExecuteHandler {
  event_handler: Arc<Box<dyn EventHandler + Send + Sync>>,
  exec_rx: ThreadSafetyReceiver<ExecEvent>,
  stoped: ThreadSafety<bool>,
}

impl ExecuteHandler {
  fn new(
    event_handler: Arc<Box<dyn EventHandler + Send + Sync>>,
    exec_rx: ThreadSafetyReceiver<ExecEvent>,
  ) -> Self {
    Self {
      event_handler,
      exec_rx,
      stoped: ThreadSafety::default(),
    }
  }

  fn run(self) -> Self {
    let exec_rx = Arc::clone(&self.exec_rx);
    let event_handler = Arc::clone(&self.event_handler);
    let stoped = Arc::clone(&self.stoped);

    let future = async move {
      loop {
        {
          // we need make sure drop the lock imediately
          let stoped = stoped.lock().await;
          if *stoped {
            break;
          }
        }

        if let Some(event) = exec_rx.lock().await.recv().await {
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
      }
    };

    tokio::spawn(future);

    self
  }

  async fn stop(&self) {
    let mut stoped = self.stoped.lock().await;
    *stoped = true;
  }
}

struct ExecuteAggregateHandler {
  event_handler: Arc<Box<dyn EventHandler + Send + Sync>>,
  changed_files: ThreadSafetyFiles,
  deleted_files: ThreadSafetyFiles,
  aggregate_timeout: u64,
  exec_aggreate_rx: ThreadSafetyReceiver<ExecAggreateEvent>,
  stoped: ThreadSafety<bool>,
}

impl ExecuteAggregateHandler {
  fn new(
    event_handler: Arc<Box<dyn EventHandler + Send + Sync>>,
    exec_aggreate_rx: ThreadSafetyReceiver<ExecAggreateEvent>,
    changed_files: ThreadSafetyFiles,
    deleted_files: ThreadSafetyFiles,
    aggregate_timeout: u64,
  ) -> Self {
    Self {
      event_handler,
      exec_aggreate_rx,
      changed_files,
      deleted_files,
      aggregate_timeout,
      stoped: ThreadSafety::default(),
    }
  }

  fn run(self) -> Self {
    let exec_aggreate_rx = Arc::clone(&self.exec_aggreate_rx);
    let event_handler = Arc::clone(&self.event_handler);
    let changed_files = Arc::clone(&self.changed_files);
    let deleted_files = Arc::clone(&self.deleted_files);
    let aggregate_timeout = self.aggregate_timeout;
    let stoped = Arc::clone(&self.stoped);

    let future = async move {
      loop {
        {
          // we need make sure drop the lock imediately
          let stoped = stoped.lock().await;
          if *stoped {
            break;
          }
        }

        if let Some(event) = exec_aggreate_rx.lock().await.recv().await {
          match event {
            ExecAggreateEvent::Execute => {
              let mut changed_files = changed_files.lock().await;
              let mut deleted_files = deleted_files.lock().await;
              if changed_files.is_empty() && deleted_files.is_empty() {
                continue; // No files to process, skip execution
              }
              tokio::time::sleep(tokio::time::Duration::from_millis(aggregate_timeout)).await;

              let changed_files = std::mem::take(&mut *changed_files);
              let deleted_files = std::mem::take(&mut *deleted_files);
              if event_handler
                .on_event_handle(changed_files, deleted_files)
                .await
                .is_err()
              {
                break;
              };
              // Check if the stoped flag is set before proceeding
            }
            ExecAggreateEvent::Close => {
              break;
            }
          }
        }
      }
    };

    tokio::spawn(future);

    self
  }

  async fn stop(&self) {
    let mut stoped = self.stoped.lock().await;
    *stoped = true;
  }
}
