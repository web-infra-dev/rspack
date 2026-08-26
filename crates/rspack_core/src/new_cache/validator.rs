use std::sync::Arc;

use rspack_cacheable::cacheable;
use rspack_error::Result;
use rspack_paths::{InternedPath, InternedPathSet};

use super::snapshot::FileSystemInfo;
use crate::{
  cache::{CacheCodec, Snapshot},
  new_cache::snapshot::SnapshotValidationResult,
};

#[cacheable]
#[derive(Debug)]
struct CacheValidatorData {
  rspack_pkg_version: String,
  cache_version: String,
  build_dependencies: InternedPathSet,
  build_dependencies_snapshot: Option<Snapshot>,
}

impl CacheValidatorData {
  fn new(rspack_pkg_version: String, cache_version: String) -> Self {
    Self {
      rspack_pkg_version,
      cache_version,
      build_dependencies: Default::default(),
      build_dependencies_snapshot: None,
    }
  }

  fn has_same_version(&self, other: &Self) -> bool {
    self.rspack_pkg_version == other.rspack_pkg_version && self.cache_version == other.cache_version
  }
}

pub(super) enum CacheValidatorResult {
  Valid,
  InvalidVersion,
  InvalidBuildDependencies {
    modified_files: InternedPathSet,
    removed_files: InternedPathSet,
  },
  InvalidError,
}

#[derive(Debug)]
pub(super) struct CacheValidator {
  data: CacheValidatorData,
  codec: Arc<CacheCodec>,
  file_system_info: FileSystemInfo,
}

impl CacheValidator {
  pub(super) fn new(
    rspack_pkg_version: String,
    cache_version: String,
    codec: Arc<CacheCodec>,
    file_system_info: FileSystemInfo,
  ) -> Self {
    Self {
      data: CacheValidatorData::new(rspack_pkg_version, cache_version),
      codec,
      file_system_info,
    }
  }

  /// See webpack's persistent build snapshot validation:
  /// https://github.com/webpack/webpack/blob/ce97d583e1cd8f3e47b70737de72e91b567a8497/lib/cache/PackFileCacheStrategy.js#L1345-L1429
  pub(super) async fn validate(&mut self, data: Option<&[u8]>) -> Result<CacheValidatorResult> {
    let Some(data) = data else {
      return Ok(CacheValidatorResult::InvalidVersion);
    };
    let validator = self.codec.decode::<CacheValidatorData>(data)?;
    if !validator.has_same_version(&self.data) {
      return Ok(CacheValidatorResult::InvalidVersion);
    }
    let Some(build_dependencies_snapshot) = &validator.build_dependencies_snapshot else {
      return Ok(CacheValidatorResult::InvalidError);
    };
    let validation = self
      .file_system_info
      .check_snapshot_valid(build_dependencies_snapshot)
      .await?;
    Ok(match validation {
      SnapshotValidationResult::Valid => {
        self.data = validator;
        CacheValidatorResult::Valid
      }
      SnapshotValidationResult::Invalid {
        modified_files,
        removed_files,
      } => CacheValidatorResult::InvalidBuildDependencies {
        modified_files,
        removed_files,
      },
    })
  }

  /// See webpack's build dependency resolution and snapshot persistence:
  /// https://github.com/webpack/webpack/blob/ce97d583e1cd8f3e47b70737de72e91b567a8497/lib/cache/PackFileCacheStrategy.js#L1510-L1625
  pub(super) async fn update(
    &mut self,
    build_dependencies: impl Iterator<Item = InternedPath>,
  ) -> Result<Option<Vec<u8>>> {
    let new_build_dependencies = build_dependencies
      .filter(|path| !self.data.build_dependencies.contains(path))
      .collect::<InternedPathSet>();
    if new_build_dependencies.is_empty() && self.data.build_dependencies_snapshot.is_some() {
      return Ok(None);
    }

    let resolved = self
      .file_system_info
      .resolve_build_dependencies(new_build_dependencies.iter().cloned())
      .await;
    let snapshot = self
      .file_system_info
      .create_snapshot(
        None,
        &resolved.files,
        &resolved.contexts,
        &resolved.missing,
        self.file_system_info.build_dependencies_strategy(),
      )
      .await?;
    self.data.build_dependencies_snapshot = Some(
      if let Some(current) = self.data.build_dependencies_snapshot.take() {
        self.file_system_info.merge_snapshots(current, snapshot)
      } else {
        snapshot
      },
    );
    self.data.build_dependencies.extend(new_build_dependencies);
    Ok(Some(self.codec.encode(&self.data)?))
  }
}
