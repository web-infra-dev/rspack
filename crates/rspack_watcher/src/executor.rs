use std::sync::{Arc, Mutex as SyncMutex, MutexGuard as SyncMutexGuard};

use rspack_util::fx_hash::FxHashSet as HashSet;
use tokio::sync::{
  Mutex,
  mpsc::{self, UnboundedReceiver, UnboundedSender},
};

use super::{EventAggregateHandler, EventHandler, FsEventKind};
use crate::EventBatch;

type ThreadSafetyReceiver<T> = ThreadSafety<UnboundedReceiver<T>>;
type ThreadSafety<T> = Arc<Mutex<T>>;
type PendingFiles = Arc<SyncMutex<FilesData>>;

#[derive(Clone, Debug)]
struct AggregatedFiles {
  changed: HashSet<String>,
  deleted: HashSet<String>,
  generation: u32,
}

impl AggregatedFiles {
  fn merge(&mut self, changed: HashSet<String>, deleted: HashSet<String>) {
    for path in changed {
      self.deleted.remove(&path);
      self.changed.insert(path);
    }
    for path in deleted {
      self.changed.remove(&path);
      self.deleted.insert(path);
    }
  }
}

#[derive(Debug, Default)]
struct FilesData {
  changed: HashSet<String>,
  deleted: HashSet<String>,
  in_flight: Option<AggregatedFiles>,
  next_generation: u32,
  aggregate_scheduled: bool,
  paused: bool,
}

impl FilesData {
  fn is_empty(&self) -> bool {
    self.changed.is_empty() && self.deleted.is_empty()
  }

  fn record(&mut self, path: String, kind: FsEventKind) {
    match kind {
      FsEventKind::Change | FsEventKind::Create => {
        self.deleted.remove(&path);
        self.changed.insert(path);
      }
      FsEventKind::Remove => {
        self.changed.remove(&path);
        self.deleted.insert(path);
      }
    }
  }

  fn next_generation(&mut self) -> u32 {
    let generation = self.next_generation;
    self.next_generation = self.next_generation.wrapping_add(1);
    generation
  }

  fn claim(&mut self) -> AggregatedFiles {
    let files = AggregatedFiles {
      changed: std::mem::take(&mut self.changed),
      deleted: std::mem::take(&mut self.deleted),
      generation: self.next_generation(),
    };
    debug_assert!(self.in_flight.is_none());
    self.in_flight = Some(files.clone());
    files
  }

  fn drain(&mut self) -> AggregatedFiles {
    let generation = self.next_generation();
    let mut files = self.in_flight.take().unwrap_or_else(|| AggregatedFiles {
      changed: Default::default(),
      deleted: Default::default(),
      generation,
    });
    files.generation = generation;
    files.merge(
      std::mem::take(&mut self.changed),
      std::mem::take(&mut self.deleted),
    );
    files
  }

  fn acknowledge(&mut self, generation: u32) -> bool {
    if self
      .in_flight
      .as_ref()
      .is_some_and(|files| files.generation == generation)
    {
      self.in_flight = None;
      true
    } else {
      false
    }
  }

  fn schedule_if_needed(&mut self) -> bool {
    if self.paused || self.aggregate_scheduled || self.in_flight.is_some() || self.is_empty() {
      return false;
    }
    self.aggregate_scheduled = true;
    true
  }
}

fn lock_pending(files: &PendingFiles) -> SyncMutexGuard<'_, FilesData> {
  files
    .lock()
    .expect("pending watcher events mutex should not be poisoned")
}

/// `WatcherExecutor` is responsible for managing the execution of file system event handlers,
/// aggregating file change and delete events, and invoking the provided event handler after
/// a configurable aggregate timeout. It receives events from a channel, tracks changed and
/// deleted files, and coordinates the event handling logic.
pub struct Executor {
  aggregate_timeout: u32,
  rx: ThreadSafetyReceiver<EventBatch>,
  files_data: PendingFiles,
  exec_aggregate_tx: UnboundedSender<ExecAggregateEvent>,
  exec_aggregate_rx: ThreadSafetyReceiver<ExecAggregateEvent>,
  exec_tx: UnboundedSender<ExecEvent>,
  exec_rx: ThreadSafetyReceiver<ExecEvent>,
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

  /// Pauses aggregate delivery. Raw events continue accumulating until resume.
  pub fn pause(&self) {
    lock_pending(&self.files_data).paused = true;
  }

  /// Atomically pauses aggregate delivery and consumes its pending events.
  /// Consumed events will not be delivered to that handler later.
  pub fn take_pending_events(&self) -> (HashSet<String>, HashSet<String>, u32) {
    let mut files = lock_pending(&self.files_data);
    files.paused = true;
    files.aggregate_scheduled = false;
    let files = files.drain();
    (files.changed, files.deleted, files.generation)
  }

  pub fn acknowledge_pending_events(&self, generation: u32) {
    let should_aggregate = {
      let mut files = lock_pending(&self.files_data);
      if files.acknowledge(generation) {
        files.aggregate_scheduled = false;
      }
      files.schedule_if_needed()
    };
    if should_aggregate {
      let _ = self.exec_aggregate_tx.send(ExecAggregateEvent::Execute);
    }
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
      lock_pending(&self.files_data).aggregate_scheduled = false;
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

      let future = async move {
        while let Some(events) = rx.lock().await.recv().await {
          let should_aggregate = {
            let mut files_data = lock_pending(&files_data);
            for event in events.aggregated() {
              files_data.record(event.path.to_string_lossy().to_string(), event.kind);
            }
            files_data.schedule_if_needed()
          };

          if should_aggregate {
            let _ = exec_aggregate_tx.send(ExecAggregateEvent::Execute);
          }

          let _ = exec_tx.send(ExecEvent::Execute(events));
        }

        let _ = exec_aggregate_tx.send(ExecAggregateEvent::Close);
        let _ = exec_tx.send(ExecEvent::Close);
      };

      tokio::spawn(future);
      self.start_waiting = true;
    }

    // abort the previous handlers if they exist
    self.abort().await;

    self.run_execute_handler(event_aggregate_handler, event_handler);

    // Flush events accumulated during the pause period.
    // Without this, events that arrived while paused would sit in files_data
    // indefinitely — the event loop already processed them (added to files_data)
    // but skipped sending Execute because paused was true. No future OS event
    // will re-deliver them, so we must kick the aggregate task ourselves.
    let should_aggregate = {
      let mut files = lock_pending(&self.files_data);
      files.paused = false;
      files.schedule_if_needed()
    };
    if should_aggregate {
      let _ = self.exec_aggregate_tx.send(ExecAggregateEvent::Execute);
    }
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
      self.exec_aggregate_tx.clone(),
      self.aggregate_timeout as u64,
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
          let handle_event = |event: crate::FsEvent| {
            let path = event.path.to_string_lossy().to_string();
            match event.kind {
              super::FsEventKind::Change | super::FsEventKind::Create => {
                event_handler.on_change(path)
              }
              super::FsEventKind::Remove => event_handler.on_delete(path),
            }
          };

          match batch_events {
            EventBatch::Shared(events) => {
              for event in events {
                if handle_event(event).is_err() {
                  break;
                }
              }
            }
            EventBatch::Split { undelayed, .. } => {
              let _ = handle_event(undelayed);
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
  pending_files: PendingFiles,
  exec_aggregate_tx: UnboundedSender<ExecAggregateEvent>,
  aggregate_timeout: u64,
) -> tokio::task::JoinHandle<()> {
  let future = async move {
    loop {
      let aggregate_rx = {
        // release the lock on exec_aggregate_rx
        // and wait for the next event
        let mut exec_aggregate_rx_guard = exec_aggregate_rx.lock().await;
        match exec_aggregate_rx_guard.recv().await {
          Some(event) => event,
          None => return,
        }
      };

      if let ExecAggregateEvent::Execute = aggregate_rx {
        // Wait for the aggregate timeout before executing the handler
        tokio::time::sleep(tokio::time::Duration::from_millis(aggregate_timeout)).await;

        // Get the files to process
        let files = {
          let mut files = lock_pending(&pending_files);
          if files.paused {
            files.aggregate_scheduled = false;
            continue;
          }
          // A stale queued Execute must not replace a batch that is still
          // waiting for the JS consumer to acknowledge or drain it.
          if files.in_flight.is_some() {
            continue;
          }
          if files.is_empty() {
            files.aggregate_scheduled = false;
            continue;
          }
          files.claim()
        };

        // Call the event handler with the changed and deleted files
        let generation = files.generation;
        let defer_acknowledgement =
          event_handler.on_event_handle_with_generation(files.changed, files.deleted, generation);
        if !defer_acknowledgement {
          let should_aggregate = {
            let mut files = lock_pending(&pending_files);
            files.acknowledge(generation);
            files.aggregate_scheduled = false;
            files.schedule_if_needed()
          };
          if should_aggregate {
            let _ = exec_aggregate_tx.send(ExecAggregateEvent::Execute);
          }
        }
      }
    }
  };

  tokio::spawn(future)
}
