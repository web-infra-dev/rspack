use crate::{WatchPattern, paths::PathAccessor};

mod directories;
mod root;

/// The `Analyzer` trait defines an interface for analyzing a [`PathAccessor`]
/// and producing a set of [`WatchPattern`]s to be watched by the file system watcher.
pub(crate) trait Analyzer: Default {
  /// Analyze the given [`PathRegister`] and return a list of [`WatchTarget`]s.
  ///
  /// # Arguments
  /// * `path_accessor` - A reference to a [`PathAccessor`] that provides access to the current state of paths, directories, and missing paths.
  ///
  /// # Returns
  /// A vector of [`WatchPattern`]s representing the paths and their watch modes.
  fn analyze<'a>(&self, path_accessor: PathAccessor<'a>) -> Vec<WatchPattern>;
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub type RecommendedAnalyzer = root::WatcherRootAnalyzer;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub type RecommendedAnalyzer = directories::WatcherDirectoriesAnalyzer;
