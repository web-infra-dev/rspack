//! Internal test support for the watcher.

use std::{
  path::Path,
  sync::{Arc, Mutex},
  time::{Instant, SystemTime},
};

use notify::{Event, RecursiveMode};
use rspack_paths::InternedPath;
use rspack_util::fx_hash::{FxHashMap, FxHashSet};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::{
  EventBatch, FsEventKind, FsWatcher, FsWatcherIgnored, FsWatcherOptions, ResolvedEvents,
  analyzer::Analyzer,
  disk_watcher::{WatchBackend, handle_event},
  paths::PathManager,
  scanner::Scanner,
  trigger::Trigger,
};

/// Recorded under the aggregate-set lock to preserve acceptance and snapshot order.
#[derive(Debug)]
pub enum ExecutorEvent {
  Accepted {
    at: Instant,
    events: Vec<(String, FsEventKind)>,
  },
  Aggregated {
    at: Instant,
    changed_files: FxHashSet<String>,
    deleted_files: FxHashSet<String>,
  },
}

pub(crate) type ExecutorTrace = Arc<Mutex<Vec<ExecutorEvent>>>;

/// Read after closing the watcher to inspect its complete executor timeline.
pub fn take_executor_events(watcher: &FsWatcher) -> Vec<ExecutorEvent> {
  std::mem::take(
    &mut *watcher
      .executor_trace
      .lock()
      .expect("executor trace poisoned"),
  )
}

/// Deterministic test backend that isolates OS events for subscription tests.
#[derive(Clone, Default)]
pub struct TestWatchBackend {
  patterns: Arc<Mutex<FxHashMap<InternedPath, RecursiveMode>>>,
}

impl TestWatchBackend {
  pub fn watched_patterns(&self) -> Vec<(InternedPath, RecursiveMode)> {
    self
      .patterns
      .lock()
      .expect("test backend lock poisoned")
      .iter()
      .map(|(p, &m)| (p.clone(), m))
      .collect()
  }

  pub fn kernel_watch_count(&self) -> usize {
    self
      .patterns
      .lock()
      .expect("test backend lock poisoned")
      .len()
  }
}

struct ControlledBackendHandle {
  patterns: Arc<Mutex<FxHashMap<InternedPath, RecursiveMode>>>,
  _trigger: Arc<Trigger>,
}

impl WatchBackend for ControlledBackendHandle {
  fn watch(&mut self, path: &Path, mode: RecursiveMode) -> notify::Result<()> {
    self
      .patterns
      .lock()
      .expect("test backend lock poisoned")
      .insert(InternedPath::from(path), mode);
    Ok(())
  }

  fn unwatch(&mut self, path: &Path) -> notify::Result<()> {
    let removed = self
      .patterns
      .lock()
      .expect("test backend lock poisoned")
      .remove(&InternedPath::from(path));
    if removed.is_some() {
      Ok(())
    } else {
      Err(notify::Error::watch_not_found())
    }
  }
}

/// Creates an `FsWatcher` backed by [`TestWatchBackend`] so spontaneous OS notifications are isolated.
pub fn new_isolated_watcher(
  options: FsWatcherOptions,
  ignored: FsWatcherIgnored,
  backend: &TestWatchBackend,
) -> FsWatcher {
  let patterns = Arc::clone(&backend.patterns);
  FsWatcher::new_with_backend(options, ignored, move |trigger| {
    Box::new(ControlledBackendHandle {
      patterns,
      _trigger: trigger,
    })
  })
}

/// Associates a logical dependency path with a physical disk target path through the production
/// subscription graph before calling `watcher.watch(...)`.
pub fn bind_file_target(
  watcher: &FsWatcher,
  logical_path: InternedPath,
  target_path: InternedPath,
) {
  watcher
    .path_manager
    .bind_custom_target(logical_path, target_path);
}

/// Sends a synthetic `notify::Event` into `watcher`'s production OS event handler.
pub fn emit_native_event(watcher: &FsWatcher, event: Event) {
  let trigger = watcher.trigger.lock().expect("should lock trigger").clone();
  if let Some(trigger) = trigger {
    handle_event(event, &trigger);
  }
}

pub struct DirectoryAnalyzer {
  paths: Arc<PathManager>,
  trigger: Arc<Trigger>,
  scan_tx: UnboundedSender<EventBatch>,
  events: UnboundedReceiver<ResolvedEvents>,
}

impl Default for DirectoryAnalyzer {
  fn default() -> Self {
    let paths = Arc::new(PathManager::default().with_follow_symlinks(true));
    let (tx, events) = unbounded_channel();
    let trigger = Arc::new(Trigger::new(Arc::clone(&paths), tx));
    let scan_tx = trigger.backfill_sender();
    Self {
      paths,
      trigger,
      scan_tx,
      events,
    }
  }
}

impl DirectoryAnalyzer {
  pub async fn update(
    &self,
    files: (Vec<InternedPath>, Vec<InternedPath>),
    contexts: (Vec<InternedPath>, Vec<InternedPath>),
    missing: (Vec<InternedPath>, Vec<InternedPath>),
  ) -> rspack_error::Result<()> {
    self.paths.reset();
    self
      .paths
      .update(
        (files.0.into_iter(), files.1.into_iter()),
        (contexts.0.into_iter(), contexts.1.into_iter()),
        (missing.0.into_iter(), missing.1.into_iter()),
      )
      .await
  }

  pub fn analyze(&self) -> Vec<(InternedPath, RecursiveMode)> {
    self
      .trigger
      .analyzer
      .analyze(self.paths.access())
      .into_iter()
      .map(|pattern| (pattern.path, pattern.mode))
      .collect()
  }

  pub fn invalidate_path(&self, path: &InternedPath) {
    self.trigger.analyzer.invalidate_path(path);
  }

  pub fn on_native_event(&self, event: Event) {
    handle_event(event, &self.trigger);
  }

  pub fn on_scanned_change(&self, path: &InternedPath) {
    self.trigger.on_scanned_change(path);
  }

  pub fn scan(&self, start_time: SystemTime) {
    Scanner::new(self.scan_tx.clone(), Arc::clone(&self.paths)).scan(start_time);
  }

  pub async fn next_events(&mut self) -> Vec<(InternedPath, FsEventKind)> {
    self
      .events
      .recv()
      .await
      .expect("event processor should remain active")
      .dependencies
      .into_iter()
      .map(|event| (event.path, event.kind))
      .collect()
  }
}
