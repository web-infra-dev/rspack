#![cfg_attr(target_os = "windows", feature(windows_by_handle))]

mod analyzer;
mod disk_watcher;
mod executor;
mod ignored;
mod paths;
mod scanner;
mod time;
mod trigger;

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
pub use ignored::{FsWatcherIgnored, FsWatcherIgnoredItem, IgnoredFn};
use paths::{PathManager, links::DepKind};
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

#[derive(Default)]
pub(crate) struct ResolvedEvents {
  pub dependencies: EventBatch,
  pub raw: EventBatch,
  pub recovered: EventBatch,
  pub context_removal: Vec<InternedPath>,
}

pub trait EventAggregateHandler {
  fn on_event_handle(&self, _changed_files: HashSet<String>, _deleted_files: HashSet<String>);

  fn on_error(&self, _error: rspack_error::Error) {}
}

pub trait EventHandler {
  fn on_change(&self, _changed_file: String) -> rspack_error::Result<()> {
    Ok(())
  }

  fn on_delete(&self, _deleted_file: String) -> rspack_error::Result<()> {
    Ok(())
  }

  fn on_context_remove(&self, removed_file: String) -> rspack_error::Result<()> {
    self.on_change(removed_file)
  }
}

#[derive(Debug, Default)]
pub struct FsWatcherOptions {
  pub follow_symlinks: bool,
  pub poll_interval: Option<u32>,
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
  path_manager: Arc<PathManager>,
  aggregated: Aggregated,
  #[cfg(feature = "testing")]
  executor_trace: testing::ExecutorTrace,
}

struct FsWatcherInner {
  path_manager: Arc<PathManager>,
  disk_watcher: Arc<Mutex<DiskWatcher>>,
  executor: Executor,
  scanner: Scanner,
  analyzer: Arc<RecommendedAnalyzer>,
  trigger: Arc<Mutex<Option<Arc<Trigger>>>>,
}

impl FsWatcher {
  pub fn new(options: FsWatcherOptions, ignored: FsWatcherIgnored) -> Self {
    let (tx, rx) = mpsc::unbounded_channel();

    let path_manager =
      Arc::new(PathManager::new(ignored).with_follow_symlinks(options.follow_symlinks));
    let trigger = Arc::new(Trigger::new(Arc::clone(&path_manager), tx));
    let analyzer = Arc::clone(&trigger.analyzer);
    let disk_watcher = Arc::new(Mutex::new(DiskWatcher::new(
      options.follow_symlinks,
      options.poll_interval,
      trigger.clone(),
    )));
    trigger.set_disk_watcher(Arc::downgrade(&disk_watcher));
    let paused = Arc::new(AtomicBool::new(false));
    let executor = Executor::new(rx, options.aggregate_timeout, Arc::clone(&paused));
    let aggregated = executor.aggregated();
    #[cfg(feature = "testing")]
    let executor_trace = executor.trace();
    let scanner = Scanner::new(trigger.backfill_sender(), Arc::clone(&path_manager));
    let trigger = Arc::new(Mutex::new(Some(trigger)));

    let inner = FsWatcherInner {
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

  pub fn peek_aggregated(&self) -> (HashSet<String>, HashSet<String>) {
    let files = self.aggregated.lock().expect("aggregated sets poisoned");
    (files.changed.clone(), files.deleted.clone())
  }

  pub fn take_aggregated(&self) -> (HashSet<String>, HashSet<String>) {
    let files = std::mem::take(&mut *self.aggregated.lock().expect("aggregated sets poisoned"));
    (files.changed, files.deleted)
  }

  pub fn collect_time_info_entries(&self) -> (TimeInfoEntries, TimeInfoEntries) {
    self.path_manager.collect_time_info_entries()
  }

  #[doc(hidden)]
  pub fn index_sizes(&self) -> (usize, usize) {
    self.path_manager.index_sizes()
  }

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

  pub fn pause(&self) -> Result<()> {
    self.paused.store(true, Ordering::Relaxed);

    Ok(())
  }
}

pub(crate) fn refresh_disk_watcher(
  disk_watcher: &Mutex<DiskWatcher>,
  analyzer: &RecommendedAnalyzer,
  path_manager: &PathManager,
) -> Result<()> {
  let raw_patterns = analyzer.analyze(path_manager.watched_access());
  let normalized = normalize_watch_patterns(raw_patterns, path_manager);
  let mut guard = disk_watcher.lock().expect("disk_watcher lock poisoned");
  guard.prepare_mode_changes(&normalized);
  if let Err(error) = guard.watch(normalized.into_iter()) {
    analyzer.invalidate_all();
    return Err(error);
  }
  drop(guard);
  path_manager.reset_watched_deltas();
  Ok(())
}

fn normalize_watch_patterns(
  patterns: Vec<WatchPattern>,
  path_manager: &PathManager,
) -> Vec<WatchPattern> {
  let should_canonicalize = cfg!(any(target_os = "macos", target_os = "linux"));
  let mut merged: InternedPathMap<notify::RecursiveMode> = InternedPathMap::default();
  let mut insert = |path: InternedPath, mode: notify::RecursiveMode| {
    let path = if should_canonicalize {
      dunce::canonicalize(&path).map_or(path, InternedPath::from)
    } else {
      path
    };
    merged
      .entry(path)
      .and_modify(|cur| {
        if mode == notify::RecursiveMode::Recursive {
          *cur = mode;
        }
      })
      .or_insert(mode);
  };

  for pattern in patterns {
    insert(pattern.path, pattern.mode);
  }

  if cfg!(any(target_os = "windows", target_os = "linux")) {
    for parent in path_manager.watched_root_parents() {
      insert(parent, notify::RecursiveMode::NonRecursive);
    }
  }

  let prune_under_recursive = cfg!(all(
    not(feature = "testing"),
    any(target_os = "macos", target_os = "windows")
  ));
  let recursive_roots: InternedPathSet = if prune_under_recursive {
    merged
      .iter()
      .filter(|(_, mode)| **mode == notify::RecursiveMode::Recursive)
      .map(|(p, _)| p.clone())
      .collect()
  } else {
    InternedPathSet::default()
  };

  merged
    .into_iter()
    .filter(|(path, _)| {
      recursive_roots.is_empty()
        || !path
          .ancestors()
          .skip(1)
          .any(|ancestor| recursive_roots.contains(&InternedPath::from(ancestor)))
    })
    .map(|(path, mode)| WatchPattern { path, mode })
    .collect()
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
  async fn watch(
    &mut self,
    files: (Vec<InternedPath>, Vec<InternedPath>),
    directories: (Vec<InternedPath>, Vec<InternedPath>),
    missing: (Vec<InternedPath>, Vec<InternedPath>),
    start_time: SystemTime,
    event_aggregate_handler: Box<dyn EventAggregateHandler + Send>,
    event_handler: Box<dyn EventHandler + Send>,
  ) {
    self.path_manager.reset();

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
    self
      .disk_watcher
      .lock()
      .expect("disk_watcher lock poisoned")
      .close();
    self.scanner.close();
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
    let watch_began = SystemTime::now();
    let scan_plan = self
      .path_manager
      .update(
        (files.0.into_iter(), files.1.into_iter()),
        (directories.0.into_iter(), directories.1.into_iter()),
        (missing.0.into_iter(), missing.1.into_iter()),
      )
      .await?;

    self.record_initial_file_mtimes(&scan_plan.newly_added_files);

    refresh_disk_watcher(&self.disk_watcher, &self.analyzer, &self.path_manager)?;
    for rebound in &scan_plan.replaced_physical_dirs {
      self
        .disk_watcher
        .lock()
        .expect("disk_watcher lock poisoned")
        .rebind_prefix(rebound.as_path());
    }

    let mut scan_out = self
      .path_manager
      .scan_contexts_with_plan(start_time, &scan_plan.contexts_to_scan, false)
      .await;

    let mut guard_iters = 0;
    while (scan_out.added_watched_paths || self.path_manager.has_dirty_context_links())
      && guard_iters < 4
    {
      guard_iters += 1;
      let dirty_contexts = self.path_manager.refresh_dirty_contexts().await;
      if !dirty_contexts.is_empty() {
        scan_out
          .changed_entries
          .retain(|(_, ctx)| !dirty_contexts.iter().any(|(d, _)| d == ctx));
      }
      refresh_disk_watcher(&self.disk_watcher, &self.analyzer, &self.path_manager)?;

      let mut next_added = false;
      let mut next_attached = Vec::new();
      if !scan_out.attached_contexts.is_empty() {
        let recheck_targets: Vec<(InternedPath, bool)> = scan_out
          .attached_contexts
          .iter()
          .filter(|c| !dirty_contexts.iter().any(|(d, _)| d == *c))
          .map(|c| (c.clone(), false))
          .collect();
        if !recheck_targets.is_empty() {
          let post = self
            .path_manager
            .scan_contexts_with_plan(watch_began, &recheck_targets, true)
            .await;
          scan_out.changed_entries.extend(post.changed_entries);
          next_added |= post.added_watched_paths;
          next_attached.extend(post.attached_contexts);
        }
      }
      if !dirty_contexts.is_empty() {
        let rescan = self
          .path_manager
          .scan_contexts_with_plan(start_time, &dirty_contexts, false)
          .await;
        scan_out.changed_entries.extend(rescan.changed_entries);
        next_added |= rescan.added_watched_paths;
        next_attached.extend(rescan.attached_contexts);
      }
      scan_out.added_watched_paths = next_added;
      scan_out.attached_contexts = next_attached;
    }

    if let Some(trigger) = self.trigger.lock().expect("should lock trigger").clone() {
      for (raw_path, root_ctx) in scan_out.changed_entries {
        trigger.on_scanned_context_change(raw_path, root_ctx);
      }
    }

    self.scanner.scan_with_plan(start_time, scan_plan);

    Ok(())
  }

  fn record_initial_file_mtimes(&self, newly_added_files: &[InternedPath]) {
    for logical_path in newly_added_files {
      for item in self
        .path_manager
        .retained_items(DepKind::File, logical_path)
      {
        self.path_manager.seed_physical_baseline_if_absent(&item);
      }
      let target_ignored = self
        .path_manager
        .bound_dep(DepKind::File, logical_path)
        .is_some_and(|b| !b.target_active);
      let initial_mtime = logical_path
        .metadata()
        .or_else(|e| {
          if self.path_manager.follows_symlinks() {
            Err(e)
          } else {
            logical_path.symlink_metadata()
          }
        })
        .and_then(|m| m.modified().or_else(|_| m.created()));
      if !target_ignored && let Ok(mtime) = initial_mtime {
        self
          .path_manager
          .set_file_time_if_absent(logical_path.clone(), mtime);
      }
    }
  }
}
