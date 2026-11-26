use std::path::{Path, PathBuf};

use rspack_core::{FileSystemInfo, Snapshot, SnapshotValidationResult};

use crate::RunPatternResult;

#[derive(Debug)]
pub(super) struct CachedPatternResult {
  pub(super) results: Vec<RunPatternResult>,
  pub(super) file_dependencies: Vec<PathBuf>,
  pub(super) context_dependencies: Vec<PathBuf>,
  pub(super) snapshot: Option<Snapshot>,
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

  pub(super) async fn is_snapshot_valid(&self, file_system_info: &FileSystemInfo) -> bool {
    match &self.snapshot {
      Some(snapshot) => matches!(
        file_system_info.check_snapshot_valid(snapshot).await,
        Ok(SnapshotValidationResult::Valid)
      ),
      None => true,
    }
  }
}
