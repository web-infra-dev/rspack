use rspack_paths::ArcPath;

use crate::watcher::{manager::PathAccessor, WatchPattern};

mod directories;
mod root;

/// The `Analyzer` trait defines an interface for analyzing a [`PathRegister`]
/// and producing a set of [`WatchTarget`]s to be watched by the file system watcher.
///
/// Implementors of this trait should provide logic to determine which paths
/// should be watched, and with what recursive mode, based on the current state
/// of the path register.
///
/// The trait is bounded by `Default` to allow easy instantiation.
pub(crate) trait Analyzer<'a> {
  fn new(path_accessor: PathAccessor<'a>) -> Self;

  /// Analyze the given [`PathRegister`] and return a list of [`WatchTarget`]s.
  ///
  /// # Arguments
  /// * `register` - The path register containing all paths to consider.
  ///
  /// # Returns
  /// A vector of [`WatchTarget`]s representing the paths and their watch modes.
  fn analyze(&'a self) -> Vec<WatchPattern>;
}

fn for_each<'a>(path_accessor: &PathAccessor<'a>, mut for_each: impl FnMut(ArcPath)) {
  let all = path_accessor.all();
  for p in all {
    for_each(p);
  }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub type RecommendedAnalyzer<'a> = root::WatcherRootAnalyzer<'a>;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub type RecommendedAnalyzer<'a> = directories::WatcherRootAnalyzer<'a>;
