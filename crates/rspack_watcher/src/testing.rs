//! Internal test support for the watcher.

use std::{
  sync::{Arc, Mutex},
  time::{Instant, SystemTime},
};

use notify::{Event, RecursiveMode};
use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::{
  EventBatch, FsEventKind, FsWatcher, FsWatcherIgnored, ResolvedEvents, TimeInfoEntries,
  analyzer::Analyzer, disk_watcher::handle_event, paths::PathManager, scanner::Scanner,
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

/// Capture the explicit dependencies now and finish collecting their context members later.
pub fn begin_time_info_snapshot(
  watcher: &FsWatcher,
) -> impl FnOnce() -> (TimeInfoEntries, TimeInfoEntries) + '_ {
  let registered = watcher.path_manager.collect_registered_time_info();
  move || watcher.path_manager.collect_context_time_info(registered)
}

pub struct DirectoryAnalyzer {
  paths: Arc<PathManager>,
  trigger: Arc<Trigger>,
  scan_tx: UnboundedSender<EventBatch>,
  events: UnboundedReceiver<ResolvedEvents>,
}

impl Default for DirectoryAnalyzer {
  fn default() -> Self {
    Self::with_ignored(FsWatcherIgnored::default())
  }
}

impl DirectoryAnalyzer {
  pub fn with_ignored(ignored: FsWatcherIgnored) -> Self {
    let paths = Arc::new(PathManager::new(ignored).with_follow_symlinks(true));
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
  pub fn scan_contexts(&self) -> tokio::task::JoinHandle<Vec<InternedPath>> {
    let paths = Arc::clone(&self.paths);
    let scan = paths.added_context_scan(SystemTime::UNIX_EPOCH);
    tokio::spawn(async move {
      scan
        .run(&paths)
        .await
        .into_iter()
        .filter(|change| change.has_subscribers())
        .map(|change| change.path.clone())
        .collect()
    })
  }

  pub async fn queue_context_changes(&self) {
    self
      .queue_context_changes_since(SystemTime::UNIX_EPOCH)
      .await;
  }

  pub async fn queue_context_changes_since(&self, start: SystemTime) {
    for change in self.paths.added_context_scan(start).run(&self.paths).await {
      self.trigger.on_context_change(change);
    }
  }

  pub fn collect_time_info_entries(&self) -> (TimeInfoEntries, TimeInfoEntries) {
    self.paths.collect_time_info_entries()
  }

  pub fn hold_scanned_file(&self, root: InternedPath, path: InternedPath) -> impl FnOnce() + use<> {
    self.paths.hold_scanned_file(root, path)
  }

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
    self.next_events_with_raw().await.1
  }

  pub async fn next_events_with_raw(
    &mut self,
  ) -> (Option<InternedPath>, Vec<(InternedPath, FsEventKind)>) {
    let events = self
      .events
      .recv()
      .await
      .expect("event processor should remain active");
    let dependencies = events
      .dependencies
      .into_iter()
      .map(|event| (event.path, event.kind))
      .collect();
    (events.raw.map(|event| event.path), dependencies)
  }
}
