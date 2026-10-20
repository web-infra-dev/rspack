mod analyzer;
mod dirwatch;
mod disk_watcher;
mod executor;
mod ignored;
mod polling;
mod registry;
mod state;
mod trigger;

use std::{
  collections::VecDeque,
  future::Future,
  pin::Pin,
  sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
  },
  time::{Duration, SystemTime},
};

use analyzer::{Analyzer, RecommendedAnalyzer};
use dirwatch::WatcherManager;
use disk_watcher::DiskWatcher;
use executor::{Aggregated, Executor};
pub use ignored::{FsWatcherIgnored, IgnoredFn};
use rspack_error::Result;
use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet as HashSet;
use tokio::sync::{mpsc, oneshot};
use trigger::Trigger;

#[derive(Debug, PartialEq, Eq, Hash)]
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
  state: Arc<WatcherManager>,
  /// Shared with the executor for the same reason: `getInfo()` drains it.
  aggregated: Aggregated,
}

struct FsWatcherInner {
  state: Arc<WatcherManager>,
  disk_watcher: DiskWatcher,
  executor: Executor,
  /// Dropped on close, so no backfill is scanned or sent afterwards.
  backfill_sender: Option<mpsc::UnboundedSender<EventBatch>>,
  analyzer: RecommendedAnalyzer,
  polling: Option<polling::PollingWatcher>,
  poll_interval: Option<Duration>,
  trigger: Arc<Mutex<Option<Arc<Trigger>>>>,
}

impl FsWatcher {
  /// Creates a new [`FsWatcher`] instance with the specified options and ignored paths.
  pub fn new(options: FsWatcherOptions, ignored: FsWatcherIgnored) -> Self {
    let (tx, rx) = mpsc::unbounded_channel();

    let state = Arc::new(WatcherManager::new(ignored, options.follow_symlinks));
    let trigger = Arc::new(Trigger::new(Arc::clone(&state), tx));
    let disk_watcher = DiskWatcher::new(
      options.follow_symlinks,
      options.poll_interval,
      trigger.clone(),
      Arc::clone(&state),
    );
    let paused = Arc::new(AtomicBool::new(false));
    let executor = Executor::new(rx, options.aggregate_timeout, Arc::clone(&paused));
    let aggregated = executor.aggregated();
    let backfill_sender = Some(trigger.backfill_sender());
    let trigger = Arc::new(Mutex::new(Some(trigger)));

    let inner = FsWatcherInner {
      state: Arc::clone(&state),
      disk_watcher,
      executor,
      backfill_sender,
      analyzer: RecommendedAnalyzer::default(),
      polling: options
        .poll_interval
        .filter(|interval| *interval > 0)
        .map(|_| Default::default()),
      poll_interval: options
        .poll_interval
        .filter(|interval| *interval > 0)
        .map(|interval| Duration::from_millis(u64::from(interval))),
      trigger: Arc::clone(&trigger),
    };

    Self {
      paused,
      trigger,
      op_tx: spawn_owner_thread(inner),
      state,
      aggregated,
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
    self.state.collect_time_info_entries()
  }

  /// How many directories index recorded paths (with their entries), and how
  /// many index registered ones, so tests can check they are reclaimed.
  #[doc(hidden)]
  pub fn index_sizes(&self) -> (usize, usize) {
    self.state.index_sizes()
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
    let delay = inner.poll_interval.unwrap_or(Duration::from_secs(1));
    let timer = tokio::time::sleep(delay);
    tokio::pin!(timer);
    let mut pending_ops = VecDeque::new();
    'owner: loop {
      let op = if let Some(op) = pending_ops.pop_front() {
        op
      } else {
        tokio::select! {
          op = rx.recv() => match op {
            Some(op) => op,
            None => break,
          },
          root = inner.disk_watcher.next_root_event(), if cfg!(target_os = "linux") && !closed && inner.polling.is_none() => {
            if let Some(root) = root && let Err(error) = inner.refresh_root_watch(root) {
              tracing::error!(target: "rspack_watcher::fs_event", "failed to restore directory watch: {error}");
            }
            continue;
          }
          _ = &mut timer, if !closed && inner.polling.is_some() => {
            let pending = {
              let poll = inner.poll();
              tokio::pin!(poll);
              loop {
                tokio::select! {
                  _ = &mut poll => break None,
                  op = rx.recv() => match op {
                    Some(op @ WatcherOp::Close { .. }) => break Some(op),
                    Some(op) => pending_ops.push_back(op),
                    None => break 'owner,
                  },
                }
              }
            };
            timer.as_mut().reset(tokio::time::Instant::now() + delay);
            match pending {
              Some(op) => op,
              None => continue,
            }
          }
        }
      };
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
          inner
            .watch(
              files,
              directories,
              missing,
              start_time,
              event_aggregate_handler,
              event_handler,
            )
            .await;
          let _ = done.send(());
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
        .enable_all()
        .build()
        .expect("create watcher runtime");

      runtime.block_on(owner_loop);
    })
    .expect("spawn watcher thread");

  tx
}

impl FsWatcherInner {
  fn refresh_root_watch(&mut self, event: disk_watcher::RootEvent) -> Result<()> {
    self.analyzer.invalidate_directory(&event.path);
    if !event.appeared {
      self.disk_watcher.invalidate_root(&event.path)?;
    }
    let patterns = self.analyzer.analyze(self.state.registry.access());
    let result = self.disk_watcher.watch(patterns.into_iter(), |path| {
      self.analyzer.invalidate_directory(path);
    });
    // Attach first: processing the old root event then observes changes made
    // while inotify had no watch, including a replacement at the same path.
    if let Some(trigger) = self.trigger.lock().expect("should lock trigger").clone() {
      trigger.on_disk_event(
        &event.path,
        if event.appeared {
          event.kind
        } else {
          FsEventKind::Remove
        },
        event.appeared,
      );
    }
    result
  }

  async fn poll(&mut self) {
    let Some(polling) = &mut self.polling else {
      return;
    };
    let mut changed = polling.scan();
    let contexts = self
      .state
      .registry
      .directories
      .all
      .iter()
      .map(|path| path.clone())
      .collect();
    changed.extend(
      self
        .state
        .scan(contexts, dirwatch::ScanReason::Discovered)
        .await,
    );
    let trigger = self.trigger.lock().expect("should lock trigger").clone();
    if let Some(trigger) = trigger {
      for path in changed {
        if !self.state.registry.is_ignored_path(path.as_ref()).await {
          trigger.on_scanned_change(&path);
        }
      }
    }
  }

  async fn watch(
    &mut self,
    files: (Vec<InternedPath>, Vec<InternedPath>),
    directories: (Vec<InternedPath>, Vec<InternedPath>),
    missing: (Vec<InternedPath>, Vec<InternedPath>),
    start_time: SystemTime,
    event_aggregate_handler: Box<dyn EventAggregateHandler + Send>,
    event_handler: Box<dyn EventHandler + Send>,
  ) {
    self.state.reset();

    if let Err(e) = self
      .wait_for_event(files, directories, missing, start_time)
      .await
    {
      event_aggregate_handler.on_error(e);
      return;
    };

    self
      .executor
      .wait_for_execute(event_aggregate_handler, event_handler)
      .await;
  }

  async fn close(&mut self) -> Result<()> {
    self.disk_watcher.close();
    self.polling.take();
    self.backfill_sender.take();
    self.executor.close().await;
    self.trigger.lock().expect("should lock trigger").take();

    Ok(())
  }

  async fn wait_for_event(
    &mut self,
    files: (Vec<InternedPath>, Vec<InternedPath>),
    directories: (Vec<InternedPath>, Vec<InternedPath>),
    missing: (Vec<InternedPath>, Vec<InternedPath>),
    start_time: SystemTime,
  ) -> Result<()> {
    self.state.update(files, directories, missing).await?;

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
    self.state.record_initial_file_mtimes();

    if let Some(polling) = &mut self.polling {
      polling.update(self.state.registry.access());
    } else {
      let watch_patterns = self.analyzer.analyze(self.state.registry.access());
      self
        .disk_watcher
        .watch(watch_patterns.into_iter(), |path| {
          self.analyzer.invalidate_directory(path);
        })?;
    }
    let changed_in_contexts = self.state.scan_contexts(start_time).await;
    if let Some(trigger) = self.trigger.lock().expect("should lock trigger").clone() {
      for path in &changed_in_contexts {
        trigger.on_scanned_change(path);
      }
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
    let registry = &self.state.registry;
    let has_added_paths = !registry.files.added.is_empty()
      || !registry.directories.added.is_empty()
      || !registry.missing.added.is_empty();
    if has_added_paths && let Some(backfill_sender) = self.backfill_sender.clone() {
      let scanned = self.state.scan_registered(start_time);
      tokio::spawn(async move {
        for batch in scanned.await {
          let _ = backfill_sender.send(batch);
        }
      });
    }

    Ok(())
  }
}
