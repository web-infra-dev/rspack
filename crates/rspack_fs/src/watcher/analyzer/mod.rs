use std::path::PathBuf;

use notify::RecursiveMode;

use crate::watcher::register::PathRegister;

mod directories;
mod root;

#[derive(Debug)]
pub(crate) struct WatchInfo {
  pub path: PathBuf,
  pub mode: RecursiveMode,
}

pub(crate) trait Analyzer: Default {
  fn analyze(&self, register: &PathRegister) -> Vec<WatchInfo>;
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub type RecommendedAnalyzer = root::WatcherRootAnalyzer;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub type RecommendedAnalyzer = directories::WatcherRootAnalyzer;
