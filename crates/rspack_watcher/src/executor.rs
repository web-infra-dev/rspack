use std::sync::{
  Arc,
  atomic::{AtomicBool, AtomicU64, Ordering},
};

use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet as HashSet;
use tokio::{
  sync::{
    Mutex,
    mpsc::{self, UnboundedReceiver, UnboundedSender},
  },
  time::{Duration, Instant},
};

use super::{EventAggregateHandler, EventHandler, FsEventKind};
use crate::{EventBatch, FsEvent, ResolvedEvents};

type ThreadSafetyReceiver<T> = ThreadSafety<UnboundedReceiver<T>>;
type ThreadSafety<T> = Arc<Mutex<T>>;

/// watchpack's `aggregatedChanges` / `aggregatedRemovals`: the events that
/// have arrived since the last aggregated batch was delivered. Guarded by a
/// synchronous mutex so a paused watcher can be drained from JS.
#[derive(Debug, Default)]
pub(crate) struct FilesData {
  pub(crate) changed: HashSet<String>,
  pub(crate) deleted: HashSet<String>,
}

impl FilesData {
  fn is_empty(&self) -> bool {
    self.changed.is_empty() && self.deleted.is_empty()
  }
}

pub(crate) type Aggregated = Arc<std::sync::Mutex<FilesData>>;

struct AggregateTime {
  base: Instant,
  last_unpaused_event: AtomicU64,
  #[cfg(feature = "testing")]
  trace: crate::testing::ExecutorTrace,
}

/// `WatcherExecutor` is responsible for managing the execution of file system event handlers,
/// aggregating file change and delete events, and invoking the provided event handler after
/// a configurable aggregate timeout. It receives events from a channel, tracks changed and
/// deleted files, and coordinates the event handling logic.
pub struct Executor {
  aggregate_timeout: u32,
  rx: ThreadSafetyReceiver<ResolvedEvents>,
  files_data: Aggregated,
  exec_aggregate_tx: UnboundedSender<ExecAggregateEvent>,
  exec_aggregate_rx: ThreadSafetyReceiver<ExecAggregateEvent>,
  exec_tx: UnboundedSender<ExecEvent>,
  exec_rx: ThreadSafetyReceiver<ExecEvent>,
  paused: Arc<AtomicBool>,
  aggregate_running: Arc<AtomicBool>,
  aggregate_time: Arc<AggregateTime>,
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
  Execute(Option<FsEvent>, EventBatch, Option<InternedPath>),
  Aggregate(FilesData),
  Close,
}

impl Executor {
  /// Create a new `WatcherExecutor` with the given receiver and optional aggregate timeout.
  /// `paused` is shared with [`crate::FsWatcher`] so pausing stays a lock-free
  /// synchronous operation.
  pub fn new(
    rx: UnboundedReceiver<ResolvedEvents>,
    aggregate_timeout: Option<u32>,
    paused: Arc<AtomicBool>,
  ) -> Self {
    let (exec_aggregate_tx, exec_aggregate_rx) = mpsc::unbounded_channel::<ExecAggregateEvent>();
    let (exec_tx, exec_rx) = mpsc::unbounded_channel::<ExecEvent>();

    Self {
      start_waiting: false,
      aggregate_running: Arc::new(AtomicBool::new(false)),
      aggregate_time: Arc::new(AggregateTime {
        base: Instant::now(),
        last_unpaused_event: AtomicU64::new(0),
        #[cfg(feature = "testing")]
        trace: Default::default(),
      }),
      paused,
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

  /// The pending aggregated sets, shared with [`crate::FsWatcher`].
  pub fn aggregated(&self) -> Aggregated {
    Arc::clone(&self.files_data)
  }

  #[cfg(feature = "testing")]
  pub(crate) fn trace(&self) -> crate::testing::ExecutorTrace {
    Arc::clone(&self.aggregate_time.trace)
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
      let aggregate_time = Arc::clone(&self.aggregate_time);

      let future = async move {
        while let Some(events) = rx.lock().await.recv().await {
          // Enqueue raw delivery before allowing an aggregate snapshot to consume it.
          {
            let mut files = files_data.lock().expect("aggregated sets poisoned");
            for event in &events.dependencies {
              let path = event.path.to_string_lossy().to_string();
              match event.kind {
                FsEventKind::Change | FsEventKind::Create => {
                  files.deleted.remove(&path);
                  files.changed.insert(path);
                }
                FsEventKind::Remove => {
                  files.changed.remove(&path);
                  files.deleted.insert(path);
                }
              }
            }
            // Like Watchpack, only changes received while unpaused restart the
            // wait. An empty batch (e.g. an unwatched path removed under the
            // watch root) must not delay a pending batch.
            if !events.dependencies.is_empty() && !paused.load(Ordering::Relaxed) {
              let at = Instant::now();
              aggregate_time.last_unpaused_event.store(
                at.duration_since(aggregate_time.base).as_nanos() as u64,
                Ordering::Relaxed,
              );
              #[cfg(feature = "testing")]
              aggregate_time
                .trace
                .lock()
                .expect("executor trace poisoned")
                .push(crate::testing::ExecutorEvent::Accepted {
                  at: at.into_std(),
                  events: events
                    .dependencies
                    .iter()
                    .map(|event| (event.path.to_string_lossy().into_owned(), event.kind))
                    .collect(),
                });
            }
            let _ = exec_tx.send(ExecEvent::Execute(
              events.raw,
              events.recovered,
              events.context_removal,
            ));
          }

          if !paused.load(Ordering::Relaxed) && !aggregate_running.load(Ordering::Relaxed) {
            let _ = exec_aggregate_tx.send(ExecAggregateEvent::Execute);
          }
        }

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

    // Flush events accumulated during the pause period.
    // Without this, events that arrived while paused would sit in files_data
    // indefinitely — the event loop already processed them (added to files_data)
    // but skipped sending Execute because paused was true. No future OS event
    // will re-deliver them, so we must kick the aggregate task ourselves.
    if !self
      .files_data
      .lock()
      .expect("aggregated sets poisoned")
      .is_empty()
    {
      let _ = self.exec_aggregate_tx.send(ExecAggregateEvent::Execute);
    }
  }

  fn run_execute_handler(
    &mut self,
    event_aggregate_handler: Box<dyn EventAggregateHandler + Send>,
    event_handler: Box<dyn EventHandler + Send>,
  ) {
    self.execute_aggregate_handle = Some(create_execute_aggregate_task(
      self.exec_tx.clone(),
      Arc::clone(&self.exec_aggregate_rx),
      Arc::clone(&self.files_data),
      self.aggregate_timeout as u64,
      Arc::clone(&self.aggregate_running),
      Arc::clone(&self.aggregate_time),
    ));

    self.execute_handle = Some(create_execute_task(
      event_aggregate_handler,
      event_handler,
      Arc::clone(&self.exec_rx),
    ));
  }
}

fn create_execute_task(
  event_aggregate_handler: Box<dyn EventAggregateHandler + Send>,
  event_handler: Box<dyn EventHandler + Send>,
  exec_rx: ThreadSafetyReceiver<ExecEvent>,
) -> tokio::task::JoinHandle<()> {
  let future = async move {
    'deliveries: while let Some(exec_event) = exec_rx.lock().await.recv().await {
      match exec_event {
        ExecEvent::Execute(raw, recovered, context_removal) => {
          for event in raw.into_iter().chain(recovered) {
            // Handle each event based on its kind
            let path = event.path.to_string_lossy().to_string();
            let result = match event.kind {
              super::FsEventKind::Change | super::FsEventKind::Create => {
                event_handler.on_change(path)
              }
              super::FsEventKind::Remove => event_handler.on_delete(path),
            };
            if result.is_err() {
              continue 'deliveries;
            }
          }
          if let Some(path) = context_removal {
            let _ = event_handler.on_context_remove(path.to_string_lossy().to_string());
          }
        }
        ExecEvent::Aggregate(files) => {
          event_aggregate_handler.on_event_handle(files.changed, files.deleted);
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
  exec_tx: UnboundedSender<ExecEvent>,
  exec_aggregate_rx: ThreadSafetyReceiver<ExecAggregateEvent>,
  files: Aggregated,
  aggregate_timeout: u64,
  running: Arc<AtomicBool>,
  aggregate_time: Arc<AggregateTime>,
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
        running.store(true, Ordering::Relaxed);
        let timeout = Duration::from_millis(aggregate_timeout);
        // Keep the initial wait for a resume flush, and extend it only for
        // events received while unpaused.
        let mut deadline = Instant::now() + timeout;
        let files = loop {
          tokio::time::sleep_until(deadline).await;

          // Check the timestamp and take the batch under the same lock used
          // when recording events, so a new change cannot slip between them.
          let mut files = files.lock().expect("aggregated sets poisoned");
          let candidate = aggregate_time.base
            + Duration::from_nanos(aggregate_time.last_unpaused_event.load(Ordering::Relaxed))
            + timeout;
          if candidate > deadline {
            deadline = candidate;
            continue;
          }
          #[cfg(feature = "testing")]
          if !files.is_empty() {
            aggregate_time
              .trace
              .lock()
              .expect("executor trace poisoned")
              .push(crate::testing::ExecutorEvent::Aggregated {
                at: Instant::now().into_std(),
                changed_files: files.changed.clone(),
                deleted_files: files.deleted.clone(),
              });
          }
          break std::mem::take(&mut *files);
        };
        if files.is_empty() {
          running.store(false, Ordering::Relaxed);
          continue;
        }

        // A path is in one set only: `Trigger` reports each event by the
        // path's state on disk, and a later event replaces an earlier one.
        let _ = exec_tx.send(ExecEvent::Aggregate(files));
        running.store(false, Ordering::Relaxed);
      }
    }
  };

  tokio::spawn(future)
}
