use std::{
  sync::Arc,
  time::{Duration, Instant},
};

use rspack_cacheable::cacheable;
use rspack_error::Error;
use rspack_paths::{ArcPath, ArcPathSet};
use rspack_tasks::{get_current_dependency_id, set_current_dependency_id};

use super::{
  build_dependencies::{BuildDeps, BuildDepsValidationResult},
  codec::CacheCodec,
  snapshot::SnapshotScope,
  storage::Storage,
};
use crate::CompilationLogger;

pub const SCOPE: &str = "meta";

#[cacheable]
struct Meta {
  version: String,
  max_dependencies_id: u32,
}

#[derive(Debug)]
pub enum CacheValidationResult {
  Valid,
  InvalidVersion {
    message: &'static str,
  },
  InvalidBuildDependencies {
    modified_files: ArcPathSet,
    removed_files: ArcPathSet,
  },
  VersionError(Error),
  BuildDependenciesError(Error),
}

#[derive(Debug)]
pub struct CacheValidationReport {
  pub result: CacheValidationResult,
  pub tracked_files: Option<usize>,
  pub version_duration: Duration,
  pub build_dependencies_duration: Option<Duration>,
}

enum VersionValidationResult {
  Valid(Option<Meta>),
  Invalid { message: &'static str },
  Error(Error),
}

/// Owns every input that determines whether persistent cache artifacts are
/// compatible with the current compiler.
///
/// This mirrors webpack's PackContainer validation: the compatibility version
/// is checked first, then the build dependency snapshot, and cached artifacts
/// are restored only when both checks succeed.
#[derive(Debug)]
pub struct CacheValidation {
  codec: Arc<CacheCodec>,
  version: String,
  build_dependencies: BuildDeps,
}

impl CacheValidation {
  pub fn new(codec: Arc<CacheCodec>, version: String, build_dependencies: BuildDeps) -> Self {
    Self {
      codec,
      version,
      build_dependencies,
    }
  }

  pub async fn validate(&mut self, storage: &dyn Storage) -> CacheValidationReport {
    let version_start = Instant::now();
    let version_result = self.validate_version(storage).await;
    let version_duration = version_start.elapsed();

    let build_dependencies_start = Instant::now();
    let (result, tracked_files) = match self.build_dependencies.validate(storage).await {
      Ok(BuildDepsValidationResult::Valid { tracked_files }) => {
        let result = match version_result {
          VersionValidationResult::Valid(meta) => {
            if let Some(meta) = meta {
              Self::restore_meta(meta);
            }
            CacheValidationResult::Valid
          }
          VersionValidationResult::Invalid { message } => {
            CacheValidationResult::InvalidVersion { message }
          }
          VersionValidationResult::Error(error) => CacheValidationResult::VersionError(error),
        };
        (result, Some(tracked_files))
      }
      Ok(BuildDepsValidationResult::Invalid {
        modified_files,
        removed_files,
      }) => (
        CacheValidationResult::InvalidBuildDependencies {
          modified_files,
          removed_files,
        },
        None,
      ),
      Err(error) => (CacheValidationResult::BuildDependenciesError(error), None),
    };
    CacheValidationReport {
      result,
      tracked_files,
      version_duration,
      build_dependencies_duration: Some(build_dependencies_start.elapsed()),
    }
  }

  pub fn save(&self, storage: &mut dyn Storage) {
    let meta = Meta {
      version: self.version.clone(),
      max_dependencies_id: get_current_dependency_id(),
    };
    storage.set(
      SCOPE,
      b"default".to_vec(),
      self.codec.encode(&meta).expect("should encode success"),
    );
  }

  pub async fn add_build_dependencies(
    &mut self,
    storage: &mut dyn Storage,
    data: impl Iterator<Item = ArcPath>,
    logger: CompilationLogger,
  ) {
    self.build_dependencies.add(storage, data, logger).await;
  }

  async fn load_meta(&self, storage: &dyn Storage) -> Result<Option<Meta>, Error> {
    let Some((_, value)) = storage.load(SCOPE).await?.pop() else {
      return Ok(None);
    };
    self.codec.decode(&value).map(Some)
  }

  async fn validate_version(&self, storage: &dyn Storage) -> VersionValidationResult {
    let meta = match self.load_meta(storage).await {
      Ok(meta) => meta,
      Err(error) => return VersionValidationResult::Error(error),
    };

    if let Some(meta) = &meta
      && meta.version != self.version
    {
      return VersionValidationResult::Invalid {
        message: "persistent cache version does not match",
      };
    }

    if meta.is_none() {
      let scopes = match storage.scopes().await {
        Ok(scopes) => scopes,
        Err(error) => return VersionValidationResult::Error(error.into()),
      };
      // Loading a missing scope may create an empty META bucket, while BUILD
      // may exist before the first artifacts are saved. Any other scope cannot
      // be checked against cache.version and is unsafe to reuse.
      if scopes
        .iter()
        .any(|scope| scope != SCOPE && scope != SnapshotScope::BUILD.name())
      {
        return VersionValidationResult::Invalid {
          message: "persistent cache version is missing",
        };
      }
    }

    VersionValidationResult::Valid(meta)
  }

  fn restore_meta(meta: Meta) {
    if get_current_dependency_id() != 0 {
      panic!("The global dependency id generator is not 0 when the persistent cache is restored.");
    }
    set_current_dependency_id(meta.max_dependencies_id);
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_fs::MemoryFileSystem;
  use rspack_tasks::within_compiler_context_for_testing;

  use super::{CacheValidation, CacheValidationResult};
  use crate::cache::persistent::{
    build_dependencies::BuildDeps,
    codec::CacheCodec,
    snapshot::{Snapshot, SnapshotOptions},
    storage::MemoryStorage,
  };

  fn create_validation(fs: Arc<MemoryFileSystem>, version: &str) -> CacheValidation {
    let codec = Arc::new(CacheCodec::new(None));
    let snapshot = Arc::new(Snapshot::new(
      SnapshotOptions::default(),
      fs.clone(),
      codec.clone(),
    ));
    let build_dependencies = Vec::new();
    CacheValidation::new(
      codec,
      version.to_string(),
      BuildDeps::new(&build_dependencies, fs, snapshot),
    )
  }

  #[tokio::test]
  async fn reports_build_dependency_validation_for_version_mismatch() {
    within_compiler_context_for_testing(async {
      let fs = Arc::new(MemoryFileSystem::default());
      let old_validation = create_validation(fs.clone(), "v1");
      let mut current_validation = create_validation(fs.clone(), "v2");
      let mut storage = MemoryStorage::default();
      old_validation.save(&mut storage);

      let report = current_validation.validate(&storage).await;
      assert!(matches!(
        report.result,
        CacheValidationResult::InvalidVersion {
          message: "persistent cache version does not match"
        }
      ));
      assert_eq!(report.tracked_files, Some(0));
      assert!(report.build_dependencies_duration.is_some());
    })
    .await;
  }
}
