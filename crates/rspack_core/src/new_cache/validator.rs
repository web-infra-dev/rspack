use std::sync::{Arc, Mutex, MutexGuard};

use rspack_cacheable::cacheable;
use rspack_error::Result;
use rspack_paths::InternedPathSet;

use super::snapshot::{FileSystemInfo, Snapshot};
use crate::{
  InfrastructureLogger, Logger, cache::CacheCodec, new_cache::snapshot::SnapshotValidationResult,
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
  data: Mutex<CacheValidatorData>,
  codec: Arc<CacheCodec>,
  file_system_info: FileSystemInfo,
  logger: Arc<InfrastructureLogger>,
}

impl CacheValidator {
  pub(super) fn new(
    rspack_pkg_version: String,
    cache_version: String,
    codec: Arc<CacheCodec>,
    file_system_info: FileSystemInfo,
    logger: Arc<InfrastructureLogger>,
  ) -> Self {
    Self {
      data: Mutex::new(CacheValidatorData::new(rspack_pkg_version, cache_version)),
      codec,
      file_system_info,
      logger,
    }
  }

  fn data(&self) -> MutexGuard<'_, CacheValidatorData> {
    self.data.lock().expect("should lock")
  }

  /// See webpack's persistent build snapshot validation:
  /// https://github.com/webpack/webpack/blob/ce97d583e1cd8f3e47b70737de72e91b567a8497/lib/cache/PackFileCacheStrategy.js#L1345-L1429
  pub(super) async fn validate(&self, data: Option<&[u8]>) -> Result<CacheValidatorResult> {
    let Some(data) = data else {
      return Ok(CacheValidatorResult::InvalidError);
    };
    let validator = self.codec.decode::<CacheValidatorData>(data)?;
    if !validator.has_same_version(&self.data()) {
      return Ok(CacheValidatorResult::InvalidVersion);
    }
    let Some(build_dependencies_snapshot) = &validator.build_dependencies_snapshot else {
      return Ok(CacheValidatorResult::InvalidError);
    };
    let start = self.logger.time("check build dependencies");
    let validation = self
      .file_system_info
      .check_snapshot_valid(build_dependencies_snapshot)
      .await;
    self.logger.time_end(start);
    let validation = validation?;
    Ok(match validation {
      SnapshotValidationResult::Valid => {
        *self.data() = validator;
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
    &self,
    mut new_build_dependencies: InternedPathSet,
  ) -> Result<Option<Vec<u8>>> {
    {
      let data = self.data();
      new_build_dependencies.retain(|path| !data.build_dependencies.contains(path));
      if new_build_dependencies.is_empty() && data.build_dependencies_snapshot.is_some() {
        return Ok(None);
      }
    }

    self.logger.debug(format!(
      "Capturing build dependencies... ({} dependencies)",
      new_build_dependencies.len()
    ));
    let start = self.logger.time("resolve build dependencies");
    let resolved = self
      .file_system_info
      .resolve_build_dependencies(new_build_dependencies.iter().cloned())
      .await;
    self.logger.time_end(start);

    let start = self.logger.time("snapshot build dependencies");
    let snapshot = self
      .file_system_info
      .create_snapshot(
        None,
        &resolved.files,
        &resolved.contexts,
        &resolved.missing,
        self.file_system_info.build_dependencies_strategy(),
      )
      .await;
    self.logger.time_end(start);
    let snapshot = snapshot?;
    let mut data = self.data();
    data.build_dependencies_snapshot = Some(
      if let Some(current) = data.build_dependencies_snapshot.take() {
        self.file_system_info.merge_snapshots(current, snapshot)
      } else {
        snapshot
      },
    );
    data.build_dependencies.extend(new_build_dependencies);
    Ok(Some(self.codec.encode(&*data)?))
  }
}
