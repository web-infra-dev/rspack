mod analyzer;
mod disk_watcher;
mod register;
mod trigger;
mod watcher_executor;

use std::{collections::HashSet, sync::Arc};

type StdReceiver<T> = std::sync::mpsc::Receiver<T>;
type StdSender<T> = std::sync::mpsc::Sender<T>;

use analyzer::{Analyzer, RecommendedAnalyzer};
pub use register::Ignored;
use register::PathRegister;
use rspack_error::Result;
pub(crate) use trigger::{FsEvent, FsEventKind, Trigger};

use crate::watcher::analyzer::WatchInfo;

pub type IncrementalPaths = (Vec<String>, Vec<String>);

#[async_trait::async_trait]
pub trait EventHandler {
  async fn on_event_handle(&self, _changed_files: HashSet<String>, _deleted_files: HashSet<String>);
  // async fn on_change(&self, _changed_file: String) {}
  // async fn on_delete(&self, _deleted_file: String) {}
}

pub struct FsWatcherOptions {
  pub follow_symlinks: bool,
  pub poll_interval: Option<u32>,
  pub aggregate_timeout: Option<u32>,
}

pub struct FsWatcher {
  path_register: Arc<PathRegister>,
  disk_watcher: disk_watcher::DiskWatcher,
  executor: watcher_executor::WatcherExecutor,
  analyzer: RecommendedAnalyzer,
}

impl FsWatcher {
  pub fn new(options: FsWatcherOptions, ignored: Option<Box<dyn Ignored>>) -> Self {
    let path_register = Arc::new(PathRegister::new(ignored));
    let (tx, rx) = std::sync::mpsc::channel::<FsEvent>();
    let trigger = Trigger::new(Arc::clone(&path_register), tx);
    let disk_watcher = disk_watcher::DiskWatcher::new(options.follow_symlinks, trigger);
    let executor = watcher_executor::WatcherExecutor::new(rx, options.aggregate_timeout);

    Self {
      analyzer: RecommendedAnalyzer::default(),
      disk_watcher,
      executor,
      path_register,
    }
  }

  pub async fn watch(
    &mut self,
    files: IncrementalPaths,
    directories: IncrementalPaths,
    missing: IncrementalPaths,
    event_handler: Box<dyn EventHandler + Send + Sync>,
  ) -> Result<()> {
    self.wait_for_event(files, directories, missing).await?;

    self.executor.execute(event_handler).await;
    Ok(())
  }

  pub fn close(&mut self) -> Result<()> {
    // In this implementation, we don't have a specific close operation.
    // If the watcher is using a background thread, we would signal it to stop.
    // For now, we can just return Ok.
    Ok(())
  }

  async fn wait_for_event(
    &mut self,
    files: IncrementalPaths,
    directories: IncrementalPaths,
    missing: IncrementalPaths,
  ) -> Result<()> {
    self.path_register.save(files, directories, missing).await;

    let watch_info = self.analyzer.analyze(&self.path_register);
    for info in watch_info {
      let WatchInfo { ref path, mode } = info;
      self.disk_watcher.watch(path, mode)?;
    }

    Ok(())
  }
}
