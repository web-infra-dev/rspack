use std::path::{Path, PathBuf};

use crate::RunPatternResult;

#[derive(Debug, Clone)]
pub(super) struct CachedPatternResult {
  pub(super) results: Vec<RunPatternResult>,
  pub(super) file_dependencies: Vec<PathBuf>,
  pub(super) context_dependencies: Vec<PathBuf>,
}

impl CachedPatternResult {
  pub(super) fn is_invalidated<'a>(
    &self,
    mut changed_paths: impl Iterator<Item = &'a Path>,
  ) -> bool {
    changed_paths.any(|changed| {
      self
        .context_dependencies
        .iter()
        .chain(&self.file_dependencies)
        .any(|dependency| changed.starts_with(dependency) || dependency.starts_with(changed))
    })
  }
}
