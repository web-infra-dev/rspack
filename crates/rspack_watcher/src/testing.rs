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
  EventBatch, FsEventKind, FsWatcher, ResolvedEvents, analyzer::Analyzer,
  disk_watcher::handle_event, paths::PathManager, scanner::Scanner, trigger::Trigger,
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

/// A full watcher whose disk events are supplied by the test.
pub fn manual_watcher() -> FsWatcher {
  FsWatcher::with_backend(Default::default(), Default::default(), |_, _, _| {
    crate::DiskWatcher::manual()
  })
}

pub async fn watch_file_subscriptions(
  watcher: &FsWatcher,
  added: &[(InternedPath, InternedPath)],
  removed: &[(InternedPath, InternedPath)],
  aggregate: impl crate::EventAggregateHandler + Send + 'static,
  raw: impl crate::EventHandler + Send + 'static,
) {
  let bindings = |files: &[(InternedPath, InternedPath)]| {
    files
      .iter()
      .map(
        |(observed, logical)| crate::subscriptions::FileSubscription {
          observed: observed.clone(),
          logical: logical.clone(),
        },
      )
      .collect()
  };
  watcher
    .watch_subscriptions(
      (bindings(added), bindings(removed)),
      (vec![], vec![]),
      (vec![], vec![]),
      SystemTime::now(),
      Box::new(aggregate),
      Box::new(raw),
    )
    .await;
}

pub fn disk_event(watcher: &FsWatcher, path: &InternedPath, kind: FsEventKind) {
  if let Some(trigger) = watcher
    .trigger
    .lock()
    .expect("should lock trigger")
    .as_ref()
  {
    trigger.on_disk_event(path, kind, kind == FsEventKind::Create);
  }
}

pub async fn check_subscriptions(watcher: &FsWatcher) {
  let done = watcher
    .trigger
    .lock()
    .expect("should lock trigger")
    .as_ref()
    .map(|trigger| trigger.check_subscriptions());
  if let Some(done) = done {
    done.await.expect("complete subscription check");
  }
}

#[derive(Default)]
pub struct WatchInstallations(FxHashSet<crate::WatchPattern>);

impl WatchInstallations {
  pub fn update(
    &mut self,
    patterns: &[(InternedPath, RecursiveMode)],
    apply: impl FnMut(&InternedPath, Option<RecursiveMode>) -> notify::Result<()>,
  ) -> rspack_error::Result<bool> {
    crate::disk_watcher::reconcile_patterns(
      &mut self.0,
      patterns.iter().map(|(path, mode)| crate::WatchPattern {
        path: path.clone(),
        mode: *mode,
      }),
      apply,
    )
  }
}
