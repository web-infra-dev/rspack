mod analyzer;
mod disk_watcher;
mod executor;
mod ignored;
mod path_manager;
mod scanner;
mod trigger;

use std::{collections::HashSet, sync::Arc};

use analyzer::{Analyzer, RecommendedAnalyzer};
use disk_watcher::DiskWatcher;
use executor::Executor;
pub use ignored::{FsWatcherIgnored, Ignored};
use path_manager::PathManager;
pub use path_manager::PathUpdater;
use rspack_error::Result;
use rspack_paths::ArcPath;
use scanner::Scanner;
use tokio::sync::mpsc;
use trigger::Trigger;

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct WatchPattern {
  path: ArcPath,
  mode: notify::RecursiveMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsEventKind {
  Change,
  Remove,
  Create,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FsEvent {
  pub path: ArcPath,
  pub kind: FsEventKind,
}

pub trait EventAggregateHandler {
  fn on_event_handle(&self, _changed_files: HashSet<String>, _deleted_files: HashSet<String>);

  fn on_error(&self, _error: rspack_error::Error) {
    // Default implementation does nothing.
  }
}

pub trait EventHandler {
  fn on_change(&self, _changed_file: String) -> rspack_error::Result<()> {
    Ok(())
  }

  fn on_delete(&self, _deleted_file: String) -> rspack_error::Result<()> {
    Ok(())
  }
}

pub struct FsWatcherOptions {
  pub follow_symlinks: bool,
  pub poll_interval: Option<u32>,
  pub aggregate_timeout: Option<u32>,
}

pub struct FsWatcher {
  path_manager: Arc<PathManager>,
  disk_watcher: DiskWatcher,
  executor: Executor,
  scanner: Scanner,
  analyzer: RecommendedAnalyzer,
}

impl FsWatcher {
  pub fn new(options: FsWatcherOptions, ignored: FsWatcherIgnored) -> Self {
    let (tx, rx) = mpsc::unbounded_channel();

    let path_manager = Arc::new(PathManager::new(ignored));
    let trigger = Trigger::new(Arc::clone(&path_manager), tx.clone());
    let disk_watcher = DiskWatcher::new(options.follow_symlinks, options.poll_interval, trigger);
    let executor = Executor::new(rx, options.aggregate_timeout);
    let scanner = Scanner::new(tx, Arc::clone(&path_manager));

    Self {
      disk_watcher,
      executor,
      path_manager,
      scanner,
      analyzer: RecommendedAnalyzer::default(),
    }
  }

  pub async fn watch(
    &mut self,
    files: PathUpdater,
    directories: PathUpdater,
    missing: PathUpdater,
    event_aggregate_handler: Box<dyn EventAggregateHandler + Send>,
    event_handler: Box<dyn EventHandler + Send>,
  ) {
    self.path_manager.reset();
    self.scanner.scan();
    if let Err(e) = self.wait_for_event(files, directories, missing).await {
      event_aggregate_handler.on_error(e);
      return;
    };

    self
      .executor
      .wait_for_execute(event_aggregate_handler, event_handler)
      .await;
  }

  pub async fn close(&mut self) -> Result<()> {
    self.disk_watcher.close();
    self.scanner.close();
    self.executor.close().await;

    Ok(())
  }

  pub fn pause(&self) -> Result<()> {
    self.executor.pause();

    Ok(())
  }

  async fn wait_for_event(
    &mut self,
    files: PathUpdater,
    directories: PathUpdater,
    missing: PathUpdater,
  ) -> Result<()> {
    self
      .path_manager
      .update_paths(files, directories, missing)
      .await?;

    let watch_patterns = self.analyzer.analyze(self.path_manager.access());
    self.disk_watcher.watch(watch_patterns.into_iter())?;

    Ok(())
  }
}
