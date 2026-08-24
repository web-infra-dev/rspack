use std::sync::Arc;

use rspack_cacheable::cacheable;
use rspack_error::Result;
use rspack_paths::{InternedPath, InternedPathSet};

use super::snapshot::{BuildDeps, BuildDepsValidationResult, FileSystemInfo, Snapshot};
use crate::cache::persistent::codec::CacheCodec;

#[cacheable]
#[derive(Debug)]
struct CacheValidatorData {
  rspack_pkg_version: String,
  cache_version: String,
  build_dependencies: InternedPathSet,
  build_dependencies_snapshot: Snapshot,
}

impl CacheValidatorData {
  fn new(rspack_pkg_version: String, cache_version: String) -> Self {
    Self {
      rspack_pkg_version,
      cache_version,
      build_dependencies: Default::default(),
      build_dependencies_snapshot: Default::default(),
    }
  }

  fn has_same_version(&self, other: &Self) -> bool {
    self.rspack_pkg_version == other.rspack_pkg_version && self.cache_version == other.cache_version
  }
}

pub(super) enum CacheValidatorResult {
  InvalidVersion,
  Valid {
    tracked_files: usize,
  },
  InvalidBuildDependencies {
    modified_files: InternedPathSet,
    removed_files: InternedPathSet,
  },
}

#[derive(Debug)]
pub(super) struct CacheValidator {
  data: CacheValidatorData,
  codec: Arc<CacheCodec>,
  file_system_info: FileSystemInfo,
  build_deps: BuildDeps,
}

impl CacheValidator {
  pub(super) fn new(
    rspack_pkg_version: String,
    cache_version: String,
    codec: Arc<CacheCodec>,
    file_system_info: FileSystemInfo,
    build_deps: BuildDeps,
  ) -> Self {
    Self {
      data: CacheValidatorData::new(rspack_pkg_version, cache_version),
      codec,
      file_system_info,
      build_deps,
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
    let validation = self
      .build_deps
      .validate_snapshot(
        &self.file_system_info,
        &validator.build_dependencies_snapshot,
        &validator.build_dependencies,
        validator.build_dependencies.len(),
      )
      .await?;
    Ok(match validation {
      BuildDepsValidationResult::Valid { tracked_files } => {
        self.data = validator;
        CacheValidatorResult::Valid { tracked_files }
      }
      BuildDepsValidationResult::Invalid {
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
    paths: impl Iterator<Item = InternedPath>,
  ) -> Result<Vec<u8>> {
    let resolved = self
      .build_deps
      .resolve_dependencies(&self.data.build_dependencies, paths)
      .await;
    if !resolved.dependencies.is_empty() {
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
      self.data.build_dependencies_snapshot = self.file_system_info.merge_snapshots(
        std::mem::take(&mut self.data.build_dependencies_snapshot),
        snapshot,
      );
      self.data.build_dependencies.extend(resolved.dependencies);
    }
    self.codec.encode(&self.data)
  }
}
