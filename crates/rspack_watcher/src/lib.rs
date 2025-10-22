mod analyzer;
mod disk_watcher;
mod executor;
mod ignored;
mod paths;
mod scanner;
mod trigger;

use std::{sync::Arc, time::SystemTime};

use analyzer::{Analyzer, RecommendedAnalyzer};
use disk_watcher::DiskWatcher;
use executor::Executor;
pub use ignored::FsWatcherIgnored;
use paths::PathManager;
use rspack_error::Result;
use rspack_paths::ArcPath;
use rspack_util::fx_hash::FxHashSet as HashSet;
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
pub(crate) struct FsEvent {
  pub path: ArcPath,
  pub kind: FsEventKind,
}

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
#[derive(Debug)]
pub struct FsWatcherOptions {
  /// Whether to follow symbolic links when watching files.
  pub follow_symlinks: bool,

  /// The interval in milliseconds to poll for changes.
  pub poll_interval: Option<u32>,

  /// The timeout in milliseconds to aggregate events.
  pub aggregate_timeout: Option<u32>,
}

pub struct FsWatcher {
  path_manager: Arc<PathManager>,
  disk_watcher: DiskWatcher,
  executor: Executor,
  scanner: Scanner,
  analyzer: RecommendedAnalyzer,
  trigger: Option<Arc<Trigger>>,
}

impl FsWatcher {
  /// Creates a new [`FsWatcher`] instance with the specified options and ignored paths.
  pub fn new(options: FsWatcherOptions, ignored: FsWatcherIgnored) -> Self {
    let (tx, rx) = mpsc::unbounded_channel();

    let path_manager = Arc::new(PathManager::new(ignored));
    let trigger = Arc::new(Trigger::new(Arc::clone(&path_manager), tx.clone()));
    let disk_watcher = DiskWatcher::new(
      options.follow_symlinks,
      options.poll_interval,
      trigger.clone(),
    );
    let executor = Executor::new(rx, options.aggregate_timeout);
    let scanner = Scanner::new(tx, Arc::clone(&path_manager));

    Self {
      disk_watcher,
      executor,
      path_manager,
      scanner,
      analyzer: RecommendedAnalyzer::default(),
      trigger: Some(trigger),
    }
  }

  /// Starts the file system watcher.
  ///
  /// # Arguments
  /// * `files` - An tuple of iterators for files to watch (added, removed).
  /// * `directories` - An tuple of iterators for directories to watch (added, removed).
  /// * `missing` - An tuple of iterators for missing paths to watch (added, removed).
  /// * `event_aggregate_handler` - A boxed trait object for handling aggregated events.
  /// * `event_handler` - A boxed trait object for handling individual events.
  pub async fn watch(
    &mut self,
    files: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
    directories: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
    missing: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
    start_time: SystemTime,
    event_aggregate_handler: Box<dyn EventAggregateHandler + Send>,
    event_handler: Box<dyn EventHandler + Send>,
  ) {
    self.path_manager.reset();

    if let Err(e) = self.wait_for_event(files, directories, missing, start_time) {
      event_aggregate_handler.on_error(e);
      return;
    };

    self
      .executor
      .wait_for_execute(event_aggregate_handler, event_handler)
      .await;
  }

  /// Closes the file system watcher, stopping all background tasks and releasing resources.
  pub async fn close(&mut self) -> Result<()> {
    self.disk_watcher.close();
    self.scanner.close();
    self.executor.close().await;
    self.trigger.take();

    Ok(())
  }

  pub fn trigger_event(&self, path: &ArcPath, kind: FsEventKind) {
    if let Some(trigger) = &self.trigger {
      trigger.on_event(path, kind);
    }
  }

  /// Pauses the file system watcher, stopping the execution of the event loop.
  pub fn pause(&self) -> Result<()> {
    self.executor.pause();

    Ok(())
  }

  fn wait_for_event(
    &mut self,
    files: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
    directories: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
    missing: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
    start_time: SystemTime,
  ) -> Result<()> {
    self.path_manager.update(files, directories, missing)?;
    self.scanner.scan(start_time);

    let watch_patterns = self.analyzer.analyze(self.path_manager.access());
    self.disk_watcher.watch(watch_patterns.into_iter())?;

    Ok(())
  }
}
