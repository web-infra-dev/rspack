use std::sync::Arc;

use rspack_cacheable::cacheable;
use rspack_error::Result;
use rspack_paths::{ArcPath, ArcPathSet};

use super::snapshot::{
  BuildDependenciesSnapshot, BuildDeps, BuildDepsValidationResult, FileSystemInfo,
};
use crate::cache::persistent::codec::CacheCodec;

#[cacheable]
#[derive(Debug)]
struct CacheValidatorData {
  rspack_pkg_version: String,
  cache_version: String,
  max_dependencies_id: u32,
  build_dependencies: BuildDependenciesSnapshot,
}

impl CacheValidatorData {
  fn new(rspack_pkg_version: String, cache_version: String) -> Self {
    Self {
      rspack_pkg_version,
      cache_version,
      max_dependencies_id: 0,
      build_dependencies: Default::default(),
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
    modified_files: ArcPathSet,
    removed_files: ArcPathSet,
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

  pub(super) async fn validate(&mut self, data: Option<&[u8]>) -> Result<CacheValidatorResult> {
    let Some(data) = data else {
      return Ok(CacheValidatorResult::InvalidVersion);
    };
    let validator = self.codec.decode::<CacheValidatorData>(data)?;
    if !validator.has_same_version(&self.data) {
      return Ok(CacheValidatorResult::InvalidVersion);
    }
    let validation = validator
      .build_dependencies
      .validate(&self.file_system_info, &self.build_deps)
      .await;
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

  pub(super) async fn update(&mut self, paths: impl Iterator<Item = ArcPath>) -> Result<Vec<u8>> {
    self
      .data
      .build_dependencies
      .update(&self.file_system_info, &mut self.build_deps, paths)
      .await;
    self.codec.encode(&self.data)
  }

  pub(super) fn store_dependency_id(&mut self, dependency_id: u32) {
    self.data.max_dependencies_id = self.data.max_dependencies_id.max(dependency_id);
  }

  pub(super) fn restore_dependency_id(&self) -> u32 {
    self.data.max_dependencies_id
  }

  pub(super) fn encode(&self) -> Result<Vec<u8>> {
    self.codec.encode(&self.data)
  }
}
