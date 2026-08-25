use std::sync::Arc;

use rspack_cacheable::{cacheable, utils::OwnedOrRef};
use rspack_error::Result;

use super::{Cache, CacheValue, snapshot::Snapshot};
use crate::{
  AsyncDependenciesBlock, BoxDependency, BoxModule, BuildResult, Module, NormalModuleBuildState,
  OptimizationBailoutItem, ValueCacheVersions,
};

const MODULE_BUILD_CACHE_NAME: &str = "Compilation/modules";

type RestoredDependencies = Vec<BoxDependency>;
// AsyncDependenciesBlock is recursive and intentionally stays behind a pointer.
#[allow(clippy::vec_box)]
type RestoredBlocks = Vec<Box<AsyncDependenciesBlock>>;

#[cacheable]
struct CachedBuildResult<'a> {
  dependencies: Vec<OwnedOrRef<'a, BoxDependency>>,
  // AsyncDependenciesBlock is recursive and intentionally stays behind a pointer.
  #[allow(clippy::vec_box)]
  blocks: Vec<OwnedOrRef<'a, AsyncDependenciesBlock>>,
}

impl CachedBuildResult<'_> {
  fn from_build_result(build_result: &BuildResult) -> CachedBuildResult<'_> {
    CachedBuildResult {
      dependencies: build_result.dependencies.iter().map(Into::into).collect(),
      blocks: build_result
        .blocks
        .iter()
        .map(|block| block.as_ref().into())
        .collect(),
    }
  }

  fn into_parts(self) -> (RestoredDependencies, RestoredBlocks) {
    (
      self
        .dependencies
        .into_iter()
        .map(OwnedOrRef::into_owned)
        .collect(),
      self
        .blocks
        .into_iter()
        .map(|block| Box::new(block.into_owned()))
        .collect(),
    )
  }
}

#[cacheable]
#[derive(Debug)]
struct ModuleBuildCacheEntry {
  state: NormalModuleBuildState,
  snapshot: Snapshot,
  build_result: Vec<u8>,
  optimization_bailouts: Vec<OptimizationBailoutItem>,
}

impl ModuleBuildCacheEntry {
  fn from_build_result(
    build_result: &BuildResult,
    snapshot: Snapshot,
    cache: &Cache,
  ) -> Result<Option<Self>> {
    let Some(module) = build_result.module.as_normal_module() else {
      return Ok(None);
    };
    if !module.build_info().cacheable {
      return Ok(None);
    }
    let Some(codec) = cache.codec() else {
      return Ok(None);
    };
    Ok(Some(Self {
      state: module.build_state(),
      snapshot,
      build_result: codec.encode(&CachedBuildResult::from_build_result(build_result))?,
      optimization_bailouts: build_result.optimization_bailouts.clone(),
    }))
  }

  fn restore(&self, module: &mut BoxModule) -> Option<()> {
    module
      .as_normal_module_mut()?
      .restore_build_state(&self.state);
    Some(())
  }

  fn build_result_parts(&self, cache: &Cache) -> Result<RestoredModuleBuild> {
    let codec = cache
      .codec()
      .ok_or_else(|| rspack_error::error!("New cache codec is unavailable"))?;
    let cached: CachedBuildResult<'static> = codec.decode(&self.build_result)?;
    let (dependencies, blocks) = cached.into_parts();
    Ok(RestoredModuleBuild {
      dependencies,
      blocks,
      optimization_bailouts: self.optimization_bailouts.clone(),
    })
  }
}

#[derive(Debug)]
pub(crate) struct RestoredModuleBuild {
  dependencies: RestoredDependencies,
  blocks: RestoredBlocks,
  optimization_bailouts: Vec<OptimizationBailoutItem>,
}

impl RestoredModuleBuild {
  pub(crate) fn into_build_result(self, module: BoxModule) -> BuildResult {
    BuildResult {
      module,
      dependencies: self.dependencies,
      blocks: self.blocks,
      optimization_bailouts: self.optimization_bailouts,
    }
  }
}

/// Per-NormalModule build cache aligned with webpack's `Compilation/modules` cache.
#[derive(Debug, Clone)]
pub(crate) struct ModuleBuildCache {
  cache: Cache,
  value_cache_versions: Arc<ValueCacheVersions>,
}

impl ModuleBuildCache {
  pub(crate) fn new(cache: Cache, value_cache_versions: Arc<ValueCacheVersions>) -> Self {
    Self {
      cache,
      value_cache_versions,
    }
  }

  #[tracing::instrument("Cache::ModuleBuild::restore", skip_all)]
  pub(crate) async fn restore(
    &self,
    module: &mut BoxModule,
  ) -> Result<Option<RestoredModuleBuild>> {
    let item_cache = self
      .cache
      .facade(MODULE_BUILD_CACHE_NAME)
      .get_item_cache(module.identifier().as_str(), None);
    let Some(entry) = item_cache.get::<ModuleBuildCacheEntry>()? else {
      return Ok(None);
    };
    if entry.state.need_build(&self.value_cache_versions)
      || !self
        .cache
        .check_module_snapshot_valid(&entry.snapshot)
        .await?
    {
      return Ok(None);
    }

    let restored = entry.build_result_parts(&self.cache)?;
    let Some(()) = entry.restore(module) else {
      return Ok(None);
    };
    Ok(Some(restored))
  }

  #[tracing::instrument("Cache::ModuleBuild::store", skip_all)]
  pub(crate) async fn store(&self, build_result: &BuildResult, start_time: u64) -> Result<()> {
    let build_info = build_result.module.build_info();
    let Some(snapshot) = self
      .cache
      .create_module_snapshot(
        start_time,
        &build_info.file_dependencies,
        &build_info.context_dependencies,
        &build_info.missing_dependencies,
        &build_info.build_dependencies,
      )
      .await?
    else {
      return Ok(());
    };
    let Some(entry) =
      ModuleBuildCacheEntry::from_build_result(build_result, snapshot, &self.cache)?
    else {
      return Ok(());
    };
    self
      .cache
      .facade(MODULE_BUILD_CACHE_NAME)
      .get_item_cache(build_result.module.identifier().as_str(), None)
      .store(CacheValue::new(entry))
  }
}
