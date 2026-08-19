use std::sync::Arc;

use rspack_cacheable::cacheable;
use rspack_error::Result;
use rspack_paths::{ArcPath, ArcPathSet};

use super::snapshot::{BuildDeps, BuildDepsValidationResult, Snapshot, SnapshotEntry};
use crate::cache::persistent::codec::CacheCodec;

#[cacheable]
#[derive(Debug, Default)]
struct BuildDependenciesSnapshot {
  dependencies: ArcPathSet,
  snapshots: Vec<SnapshotEntry>,
}

#[cacheable]
#[derive(Debug)]
struct CacheValidatorData {
  rspack_pkg_version: String,
  cache_version: String,
  build_dependencies: BuildDependenciesSnapshot,
}

impl CacheValidatorData {
  fn new(rspack_pkg_version: String, cache_version: String) -> Self {
    Self {
      rspack_pkg_version,
      cache_version,
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
  snapshot: Snapshot,
  build_deps: BuildDeps,
}

impl CacheValidator {
  pub(super) fn new(
    rspack_pkg_version: String,
    cache_version: String,
    codec: Arc<CacheCodec>,
    snapshot: Snapshot,
    build_deps: BuildDeps,
  ) -> Self {
    Self {
      data: CacheValidatorData::new(rspack_pkg_version, cache_version),
      codec,
      snapshot,
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
    let validation = self
      .build_deps
      .validate_snapshot(
        &self.snapshot,
        &validator.build_dependencies.snapshots,
        validator.build_dependencies.dependencies.len(),
      )
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
    let added = self
      .build_deps
      .resolve_dependencies(&self.data.build_dependencies.dependencies, paths)
      .await;
    let snapshots = self.snapshot.add(added.iter().cloned()).await;
    self.data.build_dependencies.dependencies.extend(added);
    self.data.build_dependencies.snapshots.extend(snapshots);
    self.codec.encode(&self.data)
  }
}
