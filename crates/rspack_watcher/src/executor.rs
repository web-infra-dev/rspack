use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};

use rspack_util::fx_hash::FxHashSet as HashSet;
use tokio::sync::{
  Mutex,
  mpsc::{self, UnboundedReceiver, UnboundedSender},
};

use super::{EventAggregateHandler, EventHandler, FsEventKind};
use crate::EventBatch;

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
  rx: ThreadSafetyReceiver<EventBatch>,
  files_data: ThreadSafety<FilesData>,
  exec_aggregate_tx: UnboundedSender<ExecAggregateEvent>,
  exec_aggregate_rx: ThreadSafetyReceiver<ExecAggregateEvent>,
  exec_tx: UnboundedSender<ExecEvent>,
  exec_rx: ThreadSafetyReceiver<ExecEvent>,
  paused: Arc<AtomicBool>,
  aggregate_running: Arc<AtomicBool>,
  start_waiting: bool,
  execute_handle: Option<tokio::task::JoinHandle<()>>,
  execute_aggregate_handle: Option<tokio::task::JoinHandle<()>>,
}

const DEFAULT_AGGREGATE_TIMEOUT: u32 = 50; // Default timeout in milliseconds

/// `ExecEvent` represents control events for the watcher executor loop.
/// - `Execute`: Indicates that an event (change or delete) has occurred and the handler should be triggered.
/// - `Close`: Indicates that the event receiver has been closed and the executor should stop.
#[derive(Debug)]
enum ExecAggregateEvent {
  /// Trigger the execution of the event handler (e.g., after a file change or delete).
  Execute,
  /// Signal to close the executor loop (e.g., when the receiver is closed).
  Close,
}

enum ExecEvent {
  Execute(EventBatch),
  Close,
}

impl Executor {
  /// Create a new `WatcherExecutor` with the given receiver and optional aggregate timeout.
  pub fn new(rx: UnboundedReceiver<EventBatch>, aggregate_timeout: Option<u32>) -> Self {
    let (exec_aggregate_tx, exec_aggregate_rx) = mpsc::unbounded_channel::<ExecAggregateEvent>();
    let (exec_tx, exec_rx) = mpsc::unbounded_channel::<ExecEvent>();

    Self {
      start_waiting: false,
      aggregate_running: Arc::new(AtomicBool::new(false)),
      paused: Arc::new(AtomicBool::new(false)),
      rx: Arc::new(Mutex::new(rx)),
      files_data: Default::default(),
      exec_aggregate_tx,
      exec_aggregate_rx: Arc::new(Mutex::new(exec_aggregate_rx)),
      exec_rx: Arc::new(Mutex::new(exec_rx)),
      exec_tx,
      execute_aggregate_handle: None,
      execute_handle: None,
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
  async fn abort(&mut self) {
    if let Some(execute_aggregate_handle) = std::mem::take(&mut self.execute_aggregate_handle) {
      execute_aggregate_handle.abort();
      // Wait for the aggregate executor to finish
      // Awaiting a cancelled task might complete as usual if the task was already completed at the time it was cancelled, but most likely it will fail with a [cancelled] JoinError.
      // So we use Err in this case.
      if let Err(err) = execute_aggregate_handle.await {
        debug_assert!(err.is_cancelled());
      }
      self.aggregate_running.store(false, Ordering::Relaxed);
    }
    if let Some(execute_handle) = std::mem::take(&mut self.execute_handle) {
      execute_handle.abort();
      // Wait for the executor to finish
      if let Err(err) = execute_handle.await {
        debug_assert!(err.is_cancelled());
      }
    }
  }

  /// Abort all executor and close the receiver.
  pub async fn close(&mut self) {
    self.abort().await;
  }

  /// Execute the watcher executor loop.
  pub async fn wait_for_execute(
    &mut self,
    event_aggregate_handler: Box<dyn EventAggregateHandler + Send>,
    event_handler: Box<dyn EventHandler + Send>,
  ) {
    if !self.start_waiting {
      let files_data = Arc::clone(&self.files_data);

      let rx = Arc::clone(&self.rx);
      let exec_aggregate_tx = self.exec_aggregate_tx.clone();
      let exec_tx = self.exec_tx.clone();
      let paused = Arc::clone(&self.paused);
      let aggregate_running = Arc::clone(&self.aggregate_running);

      let future = async move {
        while let Some(events) = rx.lock().await.recv().await {
          println!(
            "[WATCHER_DEBUG] Executor - Received batch of {} events",
            events.len()
          );
          for event in &events {
            let path = event.path.to_string_lossy().to_string();
            match event.kind {
              FsEventKind::Change => {
                println!("[WATCHER_DEBUG] Executor - Recording change for: {}", path);
                files_data.lock().await.changed.insert(path);
              }
              FsEventKind::Remove => {
                println!(
                  "[WATCHER_DEBUG] Executor - Recording deletion for: {}",
                  path
                );
                files_data.lock().await.deleted.insert(path);
              }
              FsEventKind::Create => {
                println!(
                  "[WATCHER_DEBUG] Executor - Recording creation (as change) for: {}",
                  path
                );
                files_data.lock().await.changed.insert(path);
              }
            }
          }

          let paused_status = paused.load(Ordering::Relaxed);
          let aggregate_running_status = aggregate_running.load(Ordering::Relaxed);
          println!(
            "[WATCHER_DEBUG] Executor - Status: paused={}, aggregate_running={}",
            paused_status, aggregate_running_status
          );

          if !paused_status && !aggregate_running_status {
            println!("[WATCHER_DEBUG] Executor - Triggering aggregate execution");
            let _ = exec_aggregate_tx.send(ExecAggregateEvent::Execute);
          } else {
            println!(
              "[WATCHER_DEBUG] Executor - Skipping aggregate execution (paused or already running)"
            );
          }

          let _ = exec_tx.send(ExecEvent::Execute(events));
        }

        println!("[WATCHER_DEBUG] Executor - Event receiver closed, shutting down");
        let _ = exec_aggregate_tx.send(ExecAggregateEvent::Close);
        let _ = exec_tx.send(ExecEvent::Close);
      };

      tokio::spawn(future);
      self.start_waiting = true;
    }

    self.paused.store(false, Ordering::Relaxed);
    // abort the previous handlers if they exist
    self.abort().await;

    self.run_execute_handler(event_aggregate_handler, event_handler);
  }

  fn run_execute_handler(
    &mut self,
    event_aggregate_handler: Box<dyn EventAggregateHandler + Send>,
    event_handler: Box<dyn EventHandler + Send>,
  ) {
    self.execute_aggregate_handle = Some(create_execute_aggregate_task(
      event_aggregate_handler,
      Arc::clone(&self.exec_aggregate_rx),
      Arc::clone(&self.files_data),
      self.aggregate_timeout as u64,
      Arc::clone(&self.aggregate_running),
    ));

    self.execute_handle = Some(create_execute_task(
      event_handler,
      Arc::clone(&self.exec_rx),
    ));
  }
}

fn create_execute_task(
  event_handler: Box<dyn EventHandler + Send>,
  exec_rx: ThreadSafetyReceiver<ExecEvent>,
) -> tokio::task::JoinHandle<()> {
  let future = async move {
    while let Some(exec_event) = exec_rx.lock().await.recv().await {
      match exec_event {
        ExecEvent::Execute(batch_events) => {
          for event in batch_events {
            // Handle each event based on its kind
            let path = event.path.to_string_lossy().to_string();
            match event.kind {
              super::FsEventKind::Change | super::FsEventKind::Create => {
                if event_handler.on_change(path).is_err() {
                  break;
                }
              }
              super::FsEventKind::Remove => {
                if event_handler.on_delete(path).is_err() {
                  break;
                }
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
  tokio::spawn(future)
}

fn create_execute_aggregate_task(
  event_handler: Box<dyn EventAggregateHandler + Send>,
  exec_aggregate_rx: ThreadSafetyReceiver<ExecAggregateEvent>,
  files: ThreadSafety<FilesData>,
  aggregate_timeout: u64,
  running: Arc<AtomicBool>,
) -> tokio::task::JoinHandle<()> {
  let future = async move {
    loop {
      let aggregate_rx = {
        // release the lock on exec_aggregate_rx
        // and wait for the next event
        let mut exec_aggregate_rx_guard = exec_aggregate_rx.lock().await;
        match exec_aggregate_rx_guard.recv().await {
          Some(event) => event,
          None => {
            println!("[WATCHER_DEBUG] Executor aggregate task - Receiver closed, exiting");
            return;
          }
        }
      };

      if let ExecAggregateEvent::Execute = aggregate_rx {
        println!("[WATCHER_DEBUG] Executor aggregate task - Received execute signal");
        running.store(true, Ordering::Relaxed);
        // Wait for the aggregate timeout before executing the handler
        println!(
          "[WATCHER_DEBUG] Executor aggregate task - Waiting {}ms for event aggregation",
          aggregate_timeout
        );
        tokio::time::sleep(tokio::time::Duration::from_millis(aggregate_timeout)).await;

        // Get the files to process
        let files = {
          let mut files = files.lock().await;
          if files.is_empty() {
            println!("[WATCHER_DEBUG] Executor aggregate task - No files to process after timeout");
            running.store(false, Ordering::Relaxed);
            continue;
          }
          println!(
            "[WATCHER_DEBUG] Executor aggregate task - Processing {} changed and {} deleted files",
            files.changed.len(),
            files.deleted.len()
          );
          std::mem::take(&mut *files)
        };

        // Call the event handler with the changed and deleted files
        println!("[WATCHER_DEBUG] Executor aggregate task - Calling event_aggregate_handler");
        event_handler.on_event_handle(files.changed, files.deleted);
        println!("[WATCHER_DEBUG] Executor aggregate task - event_aggregate_handler completed");
        running.store(false, Ordering::Relaxed);
      }
    }
  };

  tokio::spawn(future)
}
