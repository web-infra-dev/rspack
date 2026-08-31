use rspack_cacheable::cacheable;
use rspack_collections::Identifiable;
use rspack_error::Result;

use crate::{
  BoxModule, BuildResult, DependencyRef, FileSystemInfo, NeedBuildContext, OptimizationBailoutItem,
  ValueCacheVersions,
  dependencies_block::CachedAsyncDependenciesBlock,
  new_cache::{CacheFacade, CacheValue},
  normal_module::CachedModule,
};

/// Cache-owned result of a completed normal module build.
///
/// This is deliberately separate from [`BuildResult`]. A build result is
/// consumed while it is installed into a module graph, whereas this type is
/// immutable and can be shared by the in-memory cache. The file-cache backend
/// serializes this value only when persistence is enabled.
#[cacheable]
#[derive(Debug)]
struct CachedBuildResult {
  module: CachedModule,
  dependencies: Vec<DependencyRef>,
  blocks: Vec<CachedAsyncDependenciesBlock>,
  optimization_bailouts: Vec<OptimizationBailoutItem>,
}

impl CachedBuildResult {
  async fn from_build_result(
    result: &mut BuildResult,
    file_system_info: &FileSystemInfo,
    build_start_time: u64,
  ) -> Result<Option<Self>> {
    let Some(module) = result.module.as_normal_module() else {
      return Ok(None);
    };
    let Some(module) = module
      .save_to_cache(file_system_info, build_start_time)
      .await?
    else {
      return Ok(None);
    };

    Ok(Some(Self {
      module,
      dependencies: result.dependencies.clone(),
      blocks: result
        .blocks
        .iter_mut()
        .map(|block| CachedAsyncDependenciesBlock::from_block(block))
        .collect(),
      optimization_bailouts: result.optimization_bailouts.clone(),
    }))
  }

  async fn recover(
    &self,
    mut module: BoxModule,
    context: &NeedBuildContext<'_>,
  ) -> Result<ModuleBuildCacheRestore> {
    let Some(normal_module) = module.as_normal_module_mut() else {
      return Ok(ModuleBuildCacheRestore::Miss(module));
    };
    if !normal_module
      .recover_from_cache(&self.module, context)
      .await?
    {
      return Ok(ModuleBuildCacheRestore::Miss(module));
    }

    Ok(ModuleBuildCacheRestore::Hit(BuildResult {
      module,
      dependencies: self.dependencies.clone(),
      blocks: self
        .blocks
        .iter()
        .map(|block| block.materialize())
        .collect(),
      optimization_bailouts: self.optimization_bailouts.clone(),
    }))
  }
}

pub(crate) enum ModuleBuildCacheRestore {
  Hit(BuildResult),
  Miss(BoxModule),
}

/// Cache for completed normal module builds.
///
/// `CacheFacade` retains `CachedBuildResult` directly in memory. The generic
/// cache backend owns filesystem encoding and decoding, so module cache users
/// never need to depend on `CacheCodec` or byte buffers.
#[derive(Debug, Clone)]
pub(crate) struct ModuleBuildCache {
  cache: CacheFacade,
}

impl ModuleBuildCache {
  pub(crate) fn new(cache: CacheFacade) -> Self {
    Self { cache }
  }

  pub(crate) async fn restore(
    &self,
    module: BoxModule,
    file_system_info: &FileSystemInfo,
    value_cache_versions: &ValueCacheVersions,
  ) -> Result<ModuleBuildCacheRestore> {
    let identifier = module.identifier();
    let Some(result) = self
      .cache
      .get::<CachedBuildResult>(identifier.as_str(), None)?
    else {
      return Ok(ModuleBuildCacheRestore::Miss(module));
    };
    result
      .recover(
        module,
        &NeedBuildContext::new(file_system_info, value_cache_versions),
      )
      .await
  }

  pub(crate) async fn store(
    &self,
    result: &mut BuildResult,
    file_system_info: &FileSystemInfo,
    build_start_time: u64,
  ) -> Result<()> {
    let Some(cached_result) =
      CachedBuildResult::from_build_result(result, file_system_info, build_start_time).await?
    else {
      return Ok(());
    };
    let identifier = result.module.identifier();
    self
      .cache
      .store(identifier.as_str(), None, CacheValue::new(cached_result))
  }
}
