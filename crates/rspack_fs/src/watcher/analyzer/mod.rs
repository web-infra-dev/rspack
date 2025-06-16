use std::path::PathBuf;

use notify::RecursiveMode;

use crate::watcher::register::PathRegister;

mod directories;
mod root;

#[derive(Debug)]
pub(crate) struct WatchTarget {
  pub path: PathBuf,
  pub mode: RecursiveMode,
}

/// The `Analyzer` trait defines an interface for analyzing a [`PathRegister`]
/// and producing a set of [`WatchTarget`]s to be watched by the file system watcher.
///
/// Implementors of this trait should provide logic to determine which paths
/// should be watched, and with what recursive mode, based on the current state
/// of the path register.
///
/// The trait is bounded by `Default` to allow easy instantiation.
pub(crate) trait Analyzer: Default {
  /// Analyze the given [`PathRegister`] and return a list of [`WatchTarget`]s.
  ///
  /// # Arguments
  /// * `register` - The path register containing all paths to consider.
  ///
  /// # Returns
  /// A vector of [`WatchTarget`]s representing the paths and their watch modes.
  fn analyze(&self, register: &PathRegister) -> Vec<WatchTarget>;
}

fn for_each(register: &PathRegister, mut for_each: impl FnMut(&PathBuf)) {
  let all = register.all();
  for p in all {
    let path = p.key();
    for_each(path);
  }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub type RecommendedAnalyzer = root::WatcherRootAnalyzer;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub type RecommendedAnalyzer = directories::WatcherRootAnalyzer;
