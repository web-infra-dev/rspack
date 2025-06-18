mod analyzer;
mod disk_watcher;
mod executor;
mod manager;
mod scanner;
mod trigger;

use std::{collections::HashSet, path::PathBuf, sync::Arc};

type StdReceiver<T> = std::sync::mpsc::Receiver<T>;
type StdSender<T> = std::sync::mpsc::Sender<T>;

use analyzer::{Analyzer, RecommendedAnalyzer};
use disk_watcher::DiskWatcher;
use executor::Executor;
use manager::PathManager;
pub use manager::{Ignored, PathUpdater};
use rspack_error::Result;
use scanner::Scanner;
use trigger::Trigger;

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct WatchPattern {
  path: PathBuf,
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
  pub path: PathBuf,
  pub kind: FsEventKind,
}

#[async_trait::async_trait]
pub trait EventHandler {
  async fn on_event_handle(
    &self,
    _changed_files: HashSet<String>,
    _deleted_files: HashSet<String>,
  ) -> Result<()>;
  async fn on_change(&self, _changed_file: String) -> Result<()> {
    Ok(())
  }
  async fn on_delete(&self, _deleted_file: String) -> Result<()> {
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
}

impl FsWatcher {
  pub fn new(options: FsWatcherOptions, ignored: Option<Box<dyn Ignored>>) -> Self {
    let (tx, rx) = std::sync::mpsc::channel::<FsEvent>();

    let path_manager = Arc::new(PathManager::new(ignored));
    let trigger = Trigger::new(Arc::clone(&path_manager), tx.clone());
    let disk_watcher = DiskWatcher::new(options.follow_symlinks, trigger);
    let executor = Executor::new(rx, options.aggregate_timeout);
    let scanner = Scanner::new(tx, Arc::clone(&path_manager));

    Self {
      disk_watcher,
      executor,
      path_manager,
      scanner,
    }
  }

  pub async fn watch(
    &mut self,
    files: PathUpdater,
    directories: PathUpdater,
    missing: PathUpdater,
    event_handler: Box<dyn EventHandler + Send + Sync>,
  ) -> Result<()> {
    self.scanner.scan();
    self.wait_for_event(files, directories, missing).await?;
    self.executor.wait_for_execute(event_handler).await;

    Ok(())
  }

  pub fn close(&mut self) -> Result<()> {
    self.disk_watcher.close();
    self.scanner.close();

    Ok(())
  }

  pub async fn pause(&self) -> Result<()> {
    self.executor.pause().await;

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
      .await;

    let analyzer = RecommendedAnalyzer::new(self.path_manager.access());
    let watch_patterns = analyzer.analyze();
    self.disk_watcher.watch(watch_patterns.into_iter())?;

    Ok(())
  }
}
