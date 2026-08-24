use std::sync::Arc;

use rspack_cacheable::{cacheable, utils::OwnedOrRef};
use rspack_error::Result;

use super::{
  CacheFacade, CacheValue,
  snapshot::{FileSystemInfo, Snapshot, SnapshotValidationResult},
};
use crate::{
  AsyncDependenciesBlock, BoxDependency, BoxModule, BuildResult, ModuleIdentifier,
  OptimizationBailoutItem, ValueCacheVersions,
  cache::{CacheCodec, SnapshotStrategyOptions},
};

/// Cache namespace of the module build results, aligned with webpack's
/// `Compilation/modules` cache.
pub(super) const MODULE_BUILD_CACHE_NAME: &str = "Compilation/modules";

/// The dependency id generator is a per-run counter, so cached modules would
/// collide with freshly created dependencies. Restoring the counter of the
/// previous run keeps both id ranges disjoint.
const DEPENDENCY_ID_COUNTER_KEY: &str = "dependencyIdCounter";

#[cacheable]
struct CachedModule<'a> {
  snapshot: OwnedOrRef<'a, Snapshot>,
  module: OwnedOrRef<'a, BoxModule>,
  dependencies: Vec<OwnedOrRef<'a, BoxDependency>>,
  blocks: Vec<OwnedOrRef<'a, AsyncDependenciesBlock>>,
  optimization_bailouts: Vec<OwnedOrRef<'a, OptimizationBailoutItem>>,
}

/// Per-module build cache backed by the shared memory and filesystem caches.
///
/// This is the equivalent of webpack's `Compilation/modules` item cache: a
/// built module is stored under its identifier together with a filesystem
/// snapshot, and a later build reuses it while the snapshot stays valid.
#[derive(Debug, Clone)]
pub struct ModuleBuildCache {
  cache: CacheFacade,
  file_system_info: FileSystemInfo,
  codec: Arc<CacheCodec>,
  strategy: SnapshotStrategyOptions,
}

impl ModuleBuildCache {
  pub(super) fn new(
    cache: CacheFacade,
    file_system_info: FileSystemInfo,
    codec: Arc<CacheCodec>,
    strategy: SnapshotStrategyOptions,
  ) -> Self {
    Self {
      cache,
      file_system_info,
      codec,
      strategy,
    }
  }

  #[tracing::instrument("Cache::ModuleBuild::get", skip_all)]
  pub async fn get(
    &self,
    identifier: &ModuleIdentifier,
    value_cache_versions: &ValueCacheVersions,
  ) -> Result<Option<BuildResult>> {
    let Some(bytes) = self.cache.get::<Vec<u8>>(identifier.as_str(), None)? else {
      return Ok(None);
    };
    let cached: CachedModule = self.codec.decode(bytes.as_slice())?;
    if cached.module.as_ref().need_build(value_cache_versions) {
      return Ok(None);
    }
    if !matches!(
      self
        .file_system_info
        .check_snapshot_valid(cached.snapshot.as_ref())
        .await?,
      SnapshotValidationResult::Valid
    ) {
      return Ok(None);
    }

    Ok(Some(BuildResult {
      module: cached.module.into_owned(),
      dependencies: cached
        .dependencies
        .into_iter()
        .map(OwnedOrRef::into_owned)
        .collect(),
      blocks: cached
        .blocks
        .into_iter()
        .map(|block| Box::new(block.into_owned()))
        .collect(),
      optimization_bailouts: cached
        .optimization_bailouts
        .into_iter()
        .map(OwnedOrRef::into_owned)
        .collect(),
    }))
  }

  /// Stores the build result exactly as `Module::build` returned it, before the
  /// module graph wires the dependency and block ids into the module, so that a
  /// restored result is a drop-in replacement for a freshly built one.
  #[tracing::instrument("Cache::ModuleBuild::store", skip_all)]
  pub async fn store(&self, build_result: &BuildResult) -> Result<()> {
    let build_info = build_result.module.build_info();
    if !build_info.cacheable {
      return Ok(());
    }

    let snapshot = self
      .file_system_info
      .create_snapshot(
        None,
        &build_info.file_dependencies,
        &build_info.context_dependencies,
        &build_info.missing_dependencies,
        self.strategy,
      )
      .await?;
    let cached = CachedModule {
      snapshot: (&snapshot).into(),
      module: (&build_result.module).into(),
      dependencies: build_result.dependencies.iter().map(Into::into).collect(),
      blocks: build_result
        .blocks
        .iter()
        .map(|block| (&**block).into())
        .collect(),
      optimization_bailouts: build_result
        .optimization_bailouts
        .iter()
        .map(Into::into)
        .collect(),
    };
    // Modules holding data that cannot be serialized, such as values owned by
    // the JS side, are simply not cached.
    let Ok(bytes) = self.codec.encode(&cached) else {
      tracing::debug!(
        "Skip caching module {} because it is not serializable",
        build_result.module.identifier()
      );
      return Ok(());
    };
    self.cache.store(
      build_result.module.identifier().as_str(),
      None,
      CacheValue::new(bytes),
    )
  }

  pub fn restore_dependency_id_counter(&self) -> Result<Option<u32>> {
    Ok(
      self
        .cache
        .get::<u32>(DEPENDENCY_ID_COUNTER_KEY, None)?
        .map(|value| *value),
    )
  }

  pub fn store_dependency_id_counter(&self, counter: u32) -> Result<()> {
    self
      .cache
      .store(DEPENDENCY_ID_COUNTER_KEY, None, CacheValue::new(counter))
  }
}
