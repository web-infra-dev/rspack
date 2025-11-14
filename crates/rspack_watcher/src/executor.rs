use std::{
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
  time::Instant,
};

use rspack_util::fx_hash::FxHashSet as HashSet;
use tokio::sync::{
  Mutex, RwLock,
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
  /// Tracks the last time an event was triggered.
  ///
  /// The aggregate event handlers are only triggered after the aggregate timeout has passed from the last event.
  ///
  /// For example, if the last event was triggered at time T, and the aggregate timeout is 100ms,
  /// the event handler will only be executed if no new events are received until time T + 100ms.
  last_changed: Arc<RwLock<Option<Instant>>>,
}

const DEFAULT_AGGREGATE_TIMEOUT: u32 = 50; // Default timeout in milliseconds

enum ExecEvent {
  Execute(EventBatch),
  Close,
}

type ExecAggregateEvent = ();

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
      last_changed: Default::default(),
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
      // let aggregate_running = Arc::clone(&self.aggregate_running);
      let last_changed = Arc::clone(&self.last_changed);

      let future = async move {
        while let Some(events) = rx.lock().await.recv().await {
          for event in &events {
            let path = event.path.to_string_lossy().to_string();
            match event.kind {
              FsEventKind::Change => {
                files_data.lock().await.changed.insert(path);
              }
              FsEventKind::Remove => {
                files_data.lock().await.deleted.insert(path);
              }
              FsEventKind::Create => {
                files_data.lock().await.changed.insert(path);
              }
            }
          }

          let paused = paused.load(Ordering::Relaxed);

          if !paused {
            last_changed.write().await.replace(Instant::now());
          }

          let _ = exec_tx.send(ExecEvent::Execute(events));
        }

        // Send close signal to both executors when the main receiver is closed
        let _ = exec_aggregate_tx.send(());
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
      Arc::clone(&self.last_changed),
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
  last_changed: Arc<RwLock<Option<Instant>>>,
) -> tokio::task::JoinHandle<()> {
  let future = async move {
    loop {
      // Wait for the signal to terminate the executor
      if exec_aggregate_rx.lock().await.try_recv().is_ok() {
        break;
      }

      let time_elapsed_since_last_change = last_changed
        .read()
        .await
        .map(|t| t.elapsed().as_millis() as u64);

      let on_timeout = if let Some(elapsed) = time_elapsed_since_last_change {
        elapsed >= aggregate_timeout
      } else {
        false
      };

      if !on_timeout {
        // Not yet timed out, wait a bit and check again
        if let Some(time_elapsed_since_last_change) = time_elapsed_since_last_change {
          debug_assert!(time_elapsed_since_last_change < aggregate_timeout);
          let wait_duration = aggregate_timeout - time_elapsed_since_last_change;
          tokio::time::sleep(tokio::time::Duration::from_millis(wait_duration)).await;
        } else {
          // No changes have been recorded yet. The minimum wait is the aggregate timeout.
          tokio::time::sleep(tokio::time::Duration::from_millis(aggregate_timeout)).await;
        }

        continue;
      }

      running.store(true, Ordering::Relaxed);
      *last_changed.write().await = None;

      // Get the files to process
      let files = {
        let mut files = files.lock().await;
        if files.is_empty() {
          running.store(false, Ordering::Relaxed);
          continue;
        }
        std::mem::take(&mut *files)
      };

      // Call the event handler with the changed and deleted files
      event_handler.on_event_handle(files.changed, files.deleted);
      running.store(false, Ordering::Relaxed);
    }
  };

  tokio::spawn(future)
}
