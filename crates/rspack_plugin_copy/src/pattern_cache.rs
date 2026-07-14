use std::path::{Path, PathBuf};

use crate::RunPatternResult;

#[derive(Debug, Clone)]
pub(super) struct CachedPatternResult {
  pub(super) results: Vec<RunPatternResult>,
  pub(super) file_dependencies: Vec<PathBuf>,
  pub(super) context_dependencies: Vec<PathBuf>,
}

impl CachedPatternResult {
  pub(super) fn is_invalidated<'a>(&self, changed_paths: impl Iterator<Item = &'a Path>) -> bool {
    let mut changed_paths = changed_paths.peekable();
    if changed_paths.peek().is_none() {
      return true;
    }

    changed_paths.any(|changed| {
      self
        .file_dependencies
        .iter()
        .chain(&self.context_dependencies)
        .any(|dependency| changed.starts_with(dependency) || dependency.starts_with(changed))
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn cached(file_dependencies: &[&str], context_dependencies: &[&str]) -> CachedPatternResult {
    CachedPatternResult {
      results: Vec::new(),
      file_dependencies: file_dependencies.iter().map(PathBuf::from).collect(),
      context_dependencies: context_dependencies.iter().map(PathBuf::from).collect(),
    }
  }

  #[test]
  fn invalidates_matching_files_and_context_descendants() {
    let cached = cached(&["/project/assets/file.txt"], &["/project/assets/glob"]);

    assert!(cached.is_invalidated([Path::new("/project/assets/file.txt")].into_iter()));
    assert!(cached.is_invalidated([Path::new("/project/assets/glob/nested/new.txt")].into_iter()));
  }

  #[test]
  fn invalidates_ancestor_directory_events_for_files_and_contexts() {
    let file = cached(&["/project/assets/nested/file.txt"], &[]);
    let context = cached(&[], &["/project/assets/nested"]);

    assert!(file.is_invalidated([Path::new("/project/assets")].into_iter()));
    assert!(context.is_invalidated([Path::new("/project/assets")].into_iter()));
  }

  #[test]
  fn reuses_only_when_unrelated_watcher_provenance_exists() {
    let cached = cached(&["/project/assets/file.txt"], &["/project/assets/glob"]);

    assert!(!cached.is_invalidated([Path::new("/project/src/index.js")].into_iter()));
    assert!(cached.is_invalidated(std::iter::empty()));
  }
}
