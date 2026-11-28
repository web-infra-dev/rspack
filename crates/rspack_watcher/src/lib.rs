mod analyzer;
mod disk_watcher;
mod executor;
mod ignored;
mod paths;
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
  sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
  },
  time::SystemTime,
};

use analyzer::{Analyzer, RecommendedAnalyzer};
use disk_watcher::DiskWatcher;
use executor::{Aggregated, Executor};
use futures::future::join_all;
pub use ignored::{FsWatcherIgnored, FsWatcherIgnoredItem, IgnoredFn};
use paths::PathManager;
use rspack_error::Result;
use rspack_paths::{InternedPath, InternedPathMap, InternedPathSet};
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

// Registration and filesystem events each run on a current-thread runtime.
const SCAN_THREADS_PER_RUNTIME: usize = 8;

const WATCHER_UNAVAILABLE: &str =
  "The watcher has been stopped and cannot accept new watch requests";

enum WatcherOp {
  Watch {
    files: (Vec<InternedPath>, Vec<InternedPath>),
    directories: (Vec<InternedPath>, Vec<InternedPath>),
    missing: (Vec<InternedPath>, Vec<InternedPath>),
    start_time: SystemTime,
    event_aggregate_handler: Box<dyn EventAggregateHandler + Send>,
    event_handler: Box<dyn EventHandler + Send>,
    done: oneshot::Sender<()>,
  },
  Close {
    done: oneshot::Sender<Result<()>>,
  },
}

pub struct FsWatcher {
  paused: Arc<AtomicBool>,
  trigger: Arc<Mutex<Option<Arc<Trigger>>>>,
  op_tx: mpsc::UnboundedSender<WatcherOp>,
  /// Shared with the owner thread's [`FsWatcherInner`], so the synchronous
  /// napi getter can read time info without awaiting the op channel.
  path_manager: Arc<PathManager>,
  /// Shared with the executor for the same reason: `getInfo()` drains it.
  aggregated: Aggregated,
  #[cfg(feature = "testing")]
  executor_trace: testing::ExecutorTrace,
}

type InitialIndex = tokio::sync::watch::Receiver<bool>;

struct FsWatcherInner {
  indexing: InternedPathMap<InitialIndex>,
  scans: tokio::task::JoinSet<()>,
  path_manager: Arc<PathManager>,
  disk_watcher: DiskWatcher,
  executor: Executor,
  scanner: Scanner,
  analyzer: Arc<RecommendedAnalyzer>,
  trigger: Arc<Mutex<Option<Arc<Trigger>>>>,
}

impl FsWatcher {
  /// Creates a new [`FsWatcher`] instance with the specified options and ignored paths.
  pub fn new(options: FsWatcherOptions, ignored: FsWatcherIgnored) -> Self {
    let (tx, rx) = mpsc::unbounded_channel();

    let path_manager =
      Arc::new(PathManager::new(ignored).with_follow_symlinks(options.follow_symlinks));
    let trigger = Arc::new(Trigger::new(Arc::clone(&path_manager), tx));
    let analyzer = Arc::clone(&trigger.analyzer);
    let disk_watcher = DiskWatcher::new(
      options.follow_symlinks,
      options.poll_interval,
      trigger.clone(),
    );
    let paused = Arc::new(AtomicBool::new(false));
    let executor = Executor::new(rx, options.aggregate_timeout, Arc::clone(&paused));
    let aggregated = executor.aggregated();
    #[cfg(feature = "testing")]
    let executor_trace = executor.trace();
    let scanner = Scanner::new(trigger.backfill_sender(), Arc::clone(&path_manager));
    let trigger = Arc::new(Mutex::new(Some(trigger)));

    let inner = FsWatcherInner {
      indexing: Default::default(),
      scans: Default::default(),
      path_manager: Arc::clone(&path_manager),
      disk_watcher,
      executor,
      scanner,
      analyzer,
      trigger: Arc::clone(&trigger),
    };

    Self {
      paused,
      trigger,
      op_tx: spawn_owner_thread(inner),
      path_manager,
      aggregated,
      #[cfg(feature = "testing")]
      executor_trace,
    }
  }

  /// watchpack's `aggregatedChanges` / `aggregatedRemovals`: the events that
  /// arrived since the last aggregated batch, typically while paused. Read,
  /// not drained, so a caller peeking at them cannot swallow the batch they
  /// are about to be delivered in.
  pub fn peek_aggregated(&self) -> (HashSet<String>, HashSet<String>) {
    let files = self.aggregated.lock().expect("aggregated sets poisoned");
    (files.changed.clone(), files.deleted.clone())
  }

  /// [`Self::peek_aggregated`], drained: for the build that folds these events in,
  /// so they are not delivered again as a batch of their own on resume.
  /// Events arriving afterwards are kept.
  pub fn take_aggregated(&self) -> (HashSet<String>, HashSet<String>) {
    let files = std::mem::take(&mut *self.aggregated.lock().expect("aggregated sets poisoned"));
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

    let send_result = self.op_tx.send(WatcherOp::Watch {
      files: (files.0.collect(), files.1.collect()),
      directories: (directories.0.collect(), directories.1.collect()),
      missing: (missing.0.collect(), missing.1.collect()),
      start_time,
      event_aggregate_handler,
      event_handler,
      done: done_tx,
    });
    if let Err(send_error) = send_result
      && let WatcherOp::Watch {
        event_aggregate_handler,
        ..
      } = send_error.0
    {
      event_aggregate_handler.on_error(rspack_error::error!("{WATCHER_UNAVAILABLE}"));
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
    self.paused.store(true, Ordering::Relaxed);

    Ok(())
  }
}

fn spawn_owner_thread(mut inner: FsWatcherInner) -> mpsc::UnboundedSender<WatcherOp> {
  let (tx, mut rx) = mpsc::unbounded_channel::<WatcherOp>();

  let owner_loop = async move {
    let mut closed = false;
    while let Some(op) = rx.recv().await {
      match op {
        WatcherOp::Watch {
          event_aggregate_handler,
          done,
          ..
        } if closed => {
          event_aggregate_handler.on_error(rspack_error::error!("{WATCHER_UNAVAILABLE}"));
          drop(done);
        }
        WatcherOp::Watch {
          files,
          directories,
          missing,
          start_time,
          event_aggregate_handler,
          event_handler,
          done,
        } => {
          if let Some(scan) = inner
            .watch(
              files,
              directories,
              missing,
              start_time,
              event_aggregate_handler,
              event_handler,
            )
            .await
          {
            inner.scans.spawn(async move {
              scan.await;
              let _ = done.send(());
            });
          }
        }
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
        .max_blocking_threads(SCAN_THREADS_PER_RUNTIME)
        .enable_all()
        .build()
        .expect("create watcher runtime");

      runtime.block_on(owner_loop);
    })
    .expect("spawn watcher thread");

  tx
}

impl FsWatcherInner {
  async fn watch(
    &mut self,
    files: (Vec<InternedPath>, Vec<InternedPath>),
    directories: (Vec<InternedPath>, Vec<InternedPath>),
    missing: (Vec<InternedPath>, Vec<InternedPath>),
    start_time: SystemTime,
    event_aggregate_handler: Box<dyn EventAggregateHandler + Send>,
    event_handler: Box<dyn EventHandler + Send>,
  ) -> Option<impl Future<Output = ()> + Send + use<>> {
    self.path_manager.reset();

    let scans = match self
      .prepare_watch(files, directories, missing, start_time)
      .await
    {
      Ok(scans) => scans,
      Err(error) => {
        event_aggregate_handler.on_error(error);
        return None;
      }
    };
    self
      .executor
      .wait_for_execute(
        event_aggregate_handler,
        event_handler,
        self.indexing.values().cloned().collect(),
      )
      .await;

    while let Some(previous) = self.scans.try_join_next() {
      previous.expect("watch scan panicked");
    }
    Some(scans)
  }

  async fn close(&mut self) -> Result<()> {
    self.path_manager.close_scans();
    self.disk_watcher.close();
    self.scans.shutdown().await;
    self.scanner.close();
    self.executor.close().await;
    self.trigger.lock().expect("should lock trigger").take();

    Ok(())
  }

  async fn prepare_watch(
    &mut self,
    files: (Vec<InternedPath>, Vec<InternedPath>),
    directories: (Vec<InternedPath>, Vec<InternedPath>),
    missing: (Vec<InternedPath>, Vec<InternedPath>),
    start_time: SystemTime,
  ) -> Result<impl Future<Output = ()> + Send + use<>> {
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
    // Scan AFTER the disk watcher is registered, not before. notify's `watch()`
    // registers the underlying inotify/FSEvents watch synchronously, so once it
    // returns every later change is delivered as a live event. Scanning here
    // closes a startup race: previously the scan ran (in spawned tasks) before
    // `disk_watcher.watch()`, so it could snapshot a file's mtime before the
    // watch was active. A change landing in that gap was seen by neither path —
    // the scan read the pre-change mtime, and the watch was not yet active to
    // deliver the event — so no rebuild was triggered. With this order, a change
    // before the watch is active is on disk by scan time (caught by
    // `mtime > start_time`), and a change after it is active is delivered live.
    let targets = self.scanner.prepare(start_time);
    let roots: Vec<_> = self
      .path_manager
      .added_context_scan(start_time)
      .into_roots()
      .collect();
    let separate: Arc<InternedPathSet> = Arc::new(roots.iter().map(|(r, _)| r.clone()).collect());
    self.indexing.retain(|root, ready| {
      root.ancestors().any(|p| {
        self
          .path_manager
          .watchers
          .is_explicit_context(&InternedPath::from(p))
      }) && !*ready.borrow()
        && ready.has_changed().is_ok()
    });
    let trigger = self
      .trigger
      .lock()
      .expect("should lock trigger")
      .clone()
      .expect("watcher is open");
    let mut scans = Vec::with_capacity(roots.len());
    for (root, scan) in roots {
      let (done, ready) = tokio::sync::watch::channel(false);
      self.indexing.insert(root, ready);
      let paths = Arc::clone(&self.path_manager);
      let trigger = Arc::clone(&trigger);
      let separate = Arc::clone(&separate);
      scans.push(async move {
        scan
          .run_roots(&paths, &separate, |events| {
            trigger.on_scanned_events(events)
          })
          .await;
        let _ = done.send(true);
      });
    }
    Ok(async move {
      tokio::join!(targets, join_all(scans));
    })
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
