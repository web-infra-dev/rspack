//! Internal test support using the directory analyzer on every platform.

use std::{sync::Arc, time::SystemTime};

use notify::{Event, RecursiveMode};
use rspack_paths::InternedPath;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::{
  EventBatch, FsEventKind, analyzer::Analyzer, disk_watcher::handle_event, paths::PathManager,
  scanner::Scanner, trigger::Trigger,
};

pub struct DirectoryAnalyzer {
  paths: Arc<PathManager>,
  trigger: Arc<Trigger>,
  scan_tx: UnboundedSender<EventBatch>,
  events: UnboundedReceiver<EventBatch>,
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
      .into_iter()
      .map(|event| (event.path, event.kind))
      .collect()
  }
}
