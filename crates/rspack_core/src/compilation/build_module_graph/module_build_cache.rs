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
/// consumed while it is installed into a module graph, whereas this type can be
/// retained by the in-memory cache. The file-cache backend serializes this
/// value only when persistence is enabled.
///
/// The structure is cache-owned, but its [`DependencyRef`] values are
/// intentionally shared with the live module graph. This matches webpack,
/// which installs the cached module object itself. In particular, lazy barrel
/// processing may unset a dependency's lazy state after installation, and that
/// decision remains visible to later cache hits so the dependency stays eager.
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
    let Some(module) = result.module.as_normal_module_mut() else {
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
/// The generic cache backend owns memory retention and filesystem encoding, so
/// module cache users never need to depend on `CacheCodec` or byte buffers.
/// Incremental make may bypass memory retention while preserving filesystem
/// cache reads and writes.
#[derive(Debug, Clone)]
pub(crate) struct ModuleBuildCache {
  cache: CacheFacade,
  use_memory_cache: bool,
}

impl ModuleBuildCache {
  pub(crate) fn new(cache: CacheFacade, use_memory_cache: bool) -> Self {
    Self {
      cache,
      use_memory_cache,
    }
  }

  pub(crate) async fn restore(
    &self,
    module: BoxModule,
    file_system_info: &FileSystemInfo,
    value_cache_versions: &ValueCacheVersions,
  ) -> Result<ModuleBuildCacheRestore> {
    let identifier = module.identifier();
    let result = if self.use_memory_cache {
      self
        .cache
        .get::<CachedBuildResult>(identifier.as_str(), None)?
    } else {
      self
        .cache
        .get_without_memory::<CachedBuildResult>(identifier.as_str(), None)?
    };
    let Some(result) = result else {
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
    let value = CacheValue::new(cached_result);
    if self.use_memory_cache {
      self.cache.store(identifier.as_str(), None, value)
    } else {
      self
        .cache
        .store_without_memory(identifier.as_str(), None, value)
    }
  }
}
