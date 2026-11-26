mod analyzer;
mod directory_watcher;
mod disk_watcher;
mod executor;
mod ignored;
mod observation;
mod paths;
mod paused;
mod scanner;
mod time;
mod trigger;
mod watcher_manager;

#[cfg(feature = "testing")]
#[doc(hidden)]
pub mod testing;

use std::{
  future::Future,
  pin::Pin,
  sync::{Arc, Mutex},
  time::SystemTime,
};

use analyzer::{Analyzer, RecommendedAnalyzer};
use disk_watcher::DiskWatcher;
use executor::{Delivery, Executor};
pub use ignored::{FsWatcherIgnored, FsWatcherIgnoredItem, IgnoredFn};
use paths::PathManager;
use rspack_error::Result;
use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet as HashSet;
use scanner::Scanner;
use tokio::sync::{mpsc, oneshot};
use trigger::Trigger;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct WatchPattern {
  path: InternedPath,
  mode: notify::RecursiveMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsEventKind {
  Change,
  Remove,
  Create,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FsEvent {
  pub path: InternedPath,
  pub kind: FsEventKind,
}

/// One value of watchpack's `TimeInfoEntries` map (`types/index.d.ts`):
/// `Entry | OnlySafeTimeEntry | ExistenceOnlyTimeEntry | null`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeInfoEntry {
  Entry {
    safe_time: u64,
    timestamp: u64,
    accuracy: u64,
  },
  OnlySafeTimeEntry {
    safe_time: u64,
  },
  /// Known to exist, with no time info (`{}`).
  ExistenceOnlyTimeEntry,
  /// A watched path absent on disk.
  Null,
}

/// watchpack's `TimeInfoEntries`, as `(path, entry)` rows.
pub type TimeInfoEntries = Vec<(String, TimeInfoEntry)>;

pub(crate) type EventBatch = Vec<FsEvent>;

#[derive(Clone)]
pub(crate) struct ResolvedEvents {
  pub dependencies: EventBatch,
  pub raw: Option<FsEvent>,
  pub recovered: EventBatch,
  pub context_removal: Option<InternedPath>,
}

/// `EventAggregateHandler` is a trait for handling aggregated file system events.
/// It provides methods to handle changes and deletions of files, as well as errors.
/// Implementors of this trait can define custom behavior for these events.
/// The default implementation does nothing for the `on_error` method.
/// This trait is intended to be used with the file system watcher to aggregate events
/// and handle them in a single place.
pub trait EventAggregateHandler {
  /// Handle a batch of file system events.
  fn on_event_handle(&self, _changed_files: HashSet<String>, _deleted_files: HashSet<String>);

  /// Handle an error that occurs during file system watching.
  fn on_error(&self, _error: rspack_error::Error) {
    // Default implementation does nothing.
  }
}

/// `EventHandler` is a trait for handling individual file system events.
/// It provides methods to handle changes and deletions of files.
pub trait EventHandler {
  /// Handle a change in a file.
  fn on_change(&self, _changed_file: String) -> rspack_error::Result<()> {
    Ok(())
  }

  /// Handle a deletion of a file.
  fn on_delete(&self, _deleted_file: String) -> rspack_error::Result<()> {
    Ok(())
  }

  /// A removed context entry changes its parent directory's contents.
  fn on_context_remove(&self, removed_file: String) -> rspack_error::Result<()> {
    self.on_change(removed_file)
  }
}

/// `FsWatcherOptions` contains options for configuring the file system watcher.
#[derive(Debug, Default)]
pub struct FsWatcherOptions {
  /// Whether to follow symbolic links when watching files.
  pub follow_symlinks: bool,

  /// The interval in milliseconds to poll for changes.
  pub poll_interval: Option<u32>,

  /// The timeout in milliseconds to aggregate events.
  pub aggregate_timeout: Option<u32>,
}

const WATCHER_UNAVAILABLE: &str =
  "The watcher has been stopped and cannot accept new watch requests";

struct WatchRequest {
  pause_revision: u64,
  files: (Vec<InternedPath>, Vec<InternedPath>),
  directories: (Vec<InternedPath>, Vec<InternedPath>),
  missing: (Vec<InternedPath>, Vec<InternedPath>),
  start_time: SystemTime,
  event_aggregate_handler: Box<dyn EventAggregateHandler + Send>,
  event_handler: Box<dyn EventHandler + Send>,
  done: oneshot::Sender<()>,
}

enum WatcherOp {
  Watch(WatchRequest),
  Close { done: oneshot::Sender<Result<()>> },
}

pub struct FsWatcher {
  delivery: Arc<Delivery>,
  trigger: Arc<Mutex<Option<Arc<Trigger>>>>,
  op_tx: mpsc::UnboundedSender<WatcherOp>,
  /// Shared with the owner thread's [`FsWatcherInner`], so the synchronous
  /// napi getter can read time info without awaiting the op channel.
  path_manager: Arc<PathManager>,
  #[cfg(feature = "testing")]
  executor_trace: testing::ExecutorTrace,
}

struct FsWatcherInner {
  path_manager: Arc<PathManager>,
  disk_watcher: DiskWatcher,
  executor: Executor,
  scanner: Scanner,
  scans: tokio::task::JoinSet<()>,
  analyzer: Arc<RecommendedAnalyzer>,
  trigger: Arc<Mutex<Option<Arc<Trigger>>>>,
}

impl FsWatcher {
  /// Creates a new [`FsWatcher`] instance with the specified options and ignored paths.
  pub fn new(options: FsWatcherOptions, ignored: FsWatcherIgnored) -> Self {
    let path_manager =
      Arc::new(PathManager::new(ignored).with_follow_symlinks(options.follow_symlinks));
    let executor = Executor::new(options.aggregate_timeout);
    let delivery = Arc::clone(&executor.delivery);
    let trigger = Arc::new(Trigger::with_delivery(
      Arc::clone(&path_manager),
      Arc::clone(&delivery),
    ));
    let analyzer = Arc::clone(&trigger.analyzer);
    let disk_watcher = DiskWatcher::new(
      options.follow_symlinks,
      options.poll_interval,
      trigger.clone(),
    );
    #[cfg(feature = "testing")]
    let executor_trace = Arc::clone(&delivery.trace);
    let scanner = Scanner::with_sink(trigger.backfill_sink(), Arc::clone(&path_manager));
    let trigger = Arc::new(Mutex::new(Some(trigger)));

    let inner = FsWatcherInner {
      path_manager: Arc::clone(&path_manager),
      disk_watcher,
      executor,
      scanner,
      scans: tokio::task::JoinSet::new(),
      analyzer,
      trigger: Arc::clone(&trigger),
    };

    Self {
      delivery,
      trigger,
      op_tx: spawn_owner_thread(inner),
      path_manager,
      #[cfg(feature = "testing")]
      executor_trace,
    }
  }

  /// watchpack's `aggregatedChanges` / `aggregatedRemovals`: the events that
  /// arrived since the last aggregated batch, typically while paused. Read,
  /// not drained, so a caller peeking at them cannot swallow the batch they
  /// are about to be delivered in.
  pub fn peek_aggregated(&self) -> (HashSet<String>, HashSet<String>) {
    let files = self.delivery.aggregated(false);
    (files.changed, files.deleted)
  }

  /// [`Self::peek_aggregated`], drained: for the build that folds these events in,
  /// so they are not delivered again as a batch of their own on resume.
  /// Events arriving afterwards are kept.
  pub fn take_aggregated(&self) -> (HashSet<String>, HashSet<String>) {
    let files = self.delivery.aggregated(true);
    (files.changed, files.deleted)
  }

  /// watchpack's `collectTimeInfoEntries`, as `(fileTimestamps, directoryTimestamps)`.
  pub fn collect_time_info_entries(&self) -> (TimeInfoEntries, TimeInfoEntries) {
    self.path_manager.collect_time_info_entries()
  }

  /// How many directories index recorded paths (with their entries), and how
  /// many index registered ones, so tests can check they are reclaimed.
  #[doc(hidden)]
  pub fn index_sizes(&self) -> (usize, usize) {
    self.path_manager.index_sizes()
  }

  /// Starts the file system watcher.
  ///
  /// # Arguments
  /// * `files` - An tuple of iterators for files to watch (added, removed).
  /// * `directories` - An tuple of iterators for directories to watch (added, removed).
  /// * `missing` - An tuple of iterators for missing paths to watch (added, removed).
  /// * `event_aggregate_handler` - A boxed trait object for handling aggregated events.
  /// * `event_handler` - A boxed trait object for handling individual events.
  pub fn watch(
    &self,
    files: (
      impl Iterator<Item = InternedPath>,
      impl Iterator<Item = InternedPath>,
    ),
    directories: (
      impl Iterator<Item = InternedPath>,
      impl Iterator<Item = InternedPath>,
    ),
    missing: (
      impl Iterator<Item = InternedPath>,
      impl Iterator<Item = InternedPath>,
    ),
    start_time: SystemTime,
    event_aggregate_handler: Box<dyn EventAggregateHandler + Send>,
    event_handler: Box<dyn EventHandler + Send>,
  ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    let (done_tx, done_rx) = oneshot::channel();

    let send_result = self.op_tx.send(WatcherOp::Watch(WatchRequest {
      pause_revision: self.delivery.pause_revision(),
      files: (files.0.collect(), files.1.collect()),
      directories: (directories.0.collect(), directories.1.collect()),
      missing: (missing.0.collect(), missing.1.collect()),
      start_time,
      event_aggregate_handler,
      event_handler,
      done: done_tx,
    }));
    if let Err(send_error) = send_result
      && let WatcherOp::Watch(request) = send_error.0
    {
      request
        .event_aggregate_handler
        .on_error(rspack_error::error!("{WATCHER_UNAVAILABLE}"));
    }

    Box::pin(async move {
      let _ = done_rx.await;
    })
  }

  /// Closes the file system watcher, stopping all background tasks and releasing resources.
  pub fn close(&self) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
    let (done_tx, done_rx) = oneshot::channel();
    let submitted = self.op_tx.send(WatcherOp::Close { done: done_tx }).is_ok();

    Box::pin(async move {
      if !submitted {
        return Ok(());
      }
      match done_rx.await {
        Ok(result) => result,
        Err(_) => Err(rspack_error::error!(
          "The watcher stopped before close completed"
        )),
      }
    })
  }

  pub fn trigger_event(&self, path: &InternedPath, kind: FsEventKind) {
    let trigger = self.trigger.lock().expect("should lock trigger").clone();
    if let Some(trigger) = trigger {
      trigger.on_event(path, kind);
    }
  }

  /// Pauses the file system watcher, stopping the execution of the event loop.
  pub fn pause(&self) -> Result<()> {
    self.delivery.pause();

    Ok(())
  }
}

fn spawn_owner_thread(mut inner: FsWatcherInner) -> mpsc::UnboundedSender<WatcherOp> {
  let (tx, mut rx) = mpsc::unbounded_channel::<WatcherOp>();

  let owner_loop = async move {
    let mut closed = false;
    while let Some(op) = rx.recv().await {
      match op {
        WatcherOp::Watch(request) if closed => {
          request
            .event_aggregate_handler
            .on_error(rspack_error::error!("{WATCHER_UNAVAILABLE}"));
        }
        WatcherOp::Watch(request) => inner.watch(request).await,
        WatcherOp::Close { done } if closed => {
          let _ = done.send(Ok(()));
        }
        WatcherOp::Close { done } => {
          let _ = done.send(inner.close().await);
          closed = true;
        }
      }
    }
  };

  std::thread::Builder::new()
    .name("rspack-fs-watcher".to_string())
    .spawn(move || {
      let runtime = tokio::runtime::Builder::new_current_thread()
        .max_blocking_threads(crate::paths::DirectoryScan::MAX_METADATA_READS)
        .enable_all()
        .build()
        .expect("create watcher runtime");

      runtime.block_on(owner_loop);
    })
    .expect("spawn watcher thread");

  tx
}

impl FsWatcherInner {
  async fn watch(&mut self, request: WatchRequest) {
    let WatchRequest {
      pause_revision,
      files,
      directories,
      missing,
      start_time,
      event_aggregate_handler,
      event_handler,
      done,
    } = request;
    let trigger = self
      .trigger
      .lock()
      .expect("should lock trigger")
      .clone()
      .expect("watcher is open");
    let cycle = self.executor.delivery.begin_watch(pause_revision);
    let transition = trigger.begin_watch();
    self.path_manager.reset();

    if let Err(e) = self.register_paths(files, directories, missing).await {
      event_aggregate_handler.on_error(e);
      return;
    };

    self
      .executor
      .watch(event_aggregate_handler, event_handler)
      .await;
    let _ = transition.send(cycle);

    // Capture this registration before accepting the next watch. Each caller waits
    // for its own scans; a blocked ignored predicate must not hold up unwatch or close.
    let targets = self.scanner.prepare(start_time);
    let contexts = self
      .path_manager
      .added_context_scan(start_time, Arc::clone(&trigger.scan_sink));
    let paths = Arc::clone(&self.path_manager);
    while let Some(previous) = self.scans.try_join_next() {
      previous.expect("watch scan panicked");
    }
    self.scans.spawn(async move {
      tokio::join!(targets, contexts.run(&paths));
      let _ = done.send(());
    });
  }

  async fn close(&mut self) -> Result<()> {
    self.disk_watcher.close();
    self.scans.shutdown().await;
    self.scanner.close();
    self.executor.close().await;
    self.trigger.lock().expect("should lock trigger").take();

    Ok(())
  }

  async fn register_paths(
    &mut self,
    files: (Vec<InternedPath>, Vec<InternedPath>),
    directories: (Vec<InternedPath>, Vec<InternedPath>),
    missing: (Vec<InternedPath>, Vec<InternedPath>),
  ) -> Result<()> {
    self
      .path_manager
      .update(
        (files.0.into_iter(), files.1.into_iter()),
        (directories.0.into_iter(), directories.1.into_iter()),
        (missing.0.into_iter(), missing.1.into_iter()),
      )
      .await?;

    // Record baseline mtimes for files added in this watch() cycle, BEFORE
    // starting the disk watcher. Synchronous so a stale FSEvent delivered
    // immediately after subscription still finds a baseline to compare
    // against.
    //
    // Only newly-added files are stat'd here. Files carried across cycles
    // already have a baseline that `has_mtime_changed` advanced atomically
    // on the previous event; re-stat'ing them on every rewatch would
    // (a) cost O(N) syscalls per rebuild on large projects, and
    // (b) re-introduce the race fixed by this PR — a user write that
    //     landed in the gap between aggregate and rewatch would have its
    //     post-write mtime become the new baseline, and the FSEvent still
    //     in flight for that write would then be suppressed as a duplicate.
    self.record_initial_file_mtimes();

    let watch_patterns = self.analyzer.analyze(self.path_manager.access());
    if let Err(error) = self.disk_watcher.watch(watch_patterns.into_iter()) {
      self.analyzer.invalidate_all();
      return Err(error);
    }
    Ok(())
  }

  fn record_initial_file_mtimes(&self) {
    let accessor = self.path_manager.access();
    // `.1` is `files.added` — the diff set populated by `update()` and
    // cleared by `reset()`. In steady state (no add/remove churn) this
    // is empty and the loop is a no-op.
    let paths: Vec<_> = accessor.files().1.iter().map(|p| p.clone()).collect();
    for path in paths {
      if let Ok(mtime) = path
        .metadata()
        .and_then(|m| m.modified().or_else(|_| m.created()))
      {
        self.path_manager.set_file_time_if_absent(path, mtime);
      }
    }
  }
}
