use std::sync::Arc;

use rspack_cacheable::cacheable;
use rspack_collections::{Identifiable, IdentifierDashMap};
use rspack_error::{Result, ToStringResultToRspackResultExt};

use crate::{
  BoxModule, BuildModuleGraphArtifact, BuildResult, DependencyRef, FileSystemInfo,
  ModuleIdentifier, NeedBuildContext, OptimizationBailoutItem, ValueCacheVersions,
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
  fn from_module_graph(
    artifact: &mut BuildModuleGraphArtifact,
    module_identifier: ModuleIdentifier,
    snapshot: crate::new_cache::Snapshot,
  ) -> Option<Self> {
    let (dependencies, blocks, optimization_bailouts) = {
      let module_graph = artifact.get_module_graph();
      let module = module_graph.module_by_identifier(&module_identifier)?;
      let dependencies = module
        .get_dependencies()
        .iter()
        .map(|dependency_id| module_graph.dependency_ref_by_id(dependency_id).clone())
        .collect();
      let blocks = module
        .get_blocks()
        .iter()
        .map(|block_id| {
          CachedAsyncDependenciesBlock::from_module_graph(
            module_graph.block_by_id_expect(block_id),
            module_graph,
          )
        })
        .collect();
      let optimization_bailouts = module_graph
        .get_optimization_bailout(&module_identifier)
        .clone();
      (dependencies, blocks, optimization_bailouts)
    };
    let module = artifact
      .get_module_graph_mut()
      .module_by_identifier_mut(&module_identifier)?
      .as_normal_module_mut()?
      .save_to_cache(snapshot)?;

    Some(Self {
      module,
      dependencies,
      blocks,
      optimization_bailouts,
    })
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
  pending: Arc<IdentifierDashMap<u64>>,
}

impl ModuleBuildCache {
  pub(crate) fn new(cache: CacheFacade) -> Self {
    Self {
      cache,
      pending: Default::default(),
    }
  }

  /// Defers publishing a built module until the build-module-graph phase has
  /// completed, so make-stage mutations are included in the cache entry.
  pub(crate) fn mark_pending(&self, module_identifier: ModuleIdentifier, build_start_time: u64) {
    self.pending.insert(module_identifier, build_start_time);
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

  /// Stores pending modules from the final make artifact.
  ///
  /// Snapshot creation remains parallel even though publication is delayed to
  /// `after_build_module_graph`. Only modules built in this phase are visited.
  pub(crate) async fn store_pending(
    &self,
    artifact: &mut BuildModuleGraphArtifact,
    file_system_info: &FileSystemInfo,
  ) -> Result<()> {
    let pending = self
      .pending
      .iter()
      .map(|entry| (*entry.key(), *entry.value()))
      .collect::<Vec<_>>();
    self.pending.clear();

    let module_graph = artifact.get_module_graph();
    let snapshots = rspack_parallel::scope::<_, Result<_>>(|token| {
      for (module_identifier, build_start_time) in pending {
        // SAFETY: the scope is awaited before the module graph is mutated.
        let task = unsafe { token.used((module_graph, file_system_info)) };
        task.spawn(move |(module_graph, file_system_info)| async move {
          let Some(module) = module_graph.module_by_identifier(&module_identifier) else {
            return Ok(None);
          };
          let Some(module) = module.as_normal_module() else {
            return Ok(None);
          };
          Ok(
            module
              .create_cache_snapshot(file_system_info, build_start_time)
              .await?
              .map(|snapshot| (module_identifier, snapshot)),
          )
        });
      }
    })
    .await
    .into_iter()
    .map(|result| result.to_rspack_result().and_then(|result| result))
    .collect::<Result<Vec<_>>>()?;

    for (module_identifier, snapshot) in snapshots.into_iter().flatten() {
      let Some(cached_result) =
        CachedBuildResult::from_module_graph(artifact, module_identifier, snapshot)
      else {
        continue;
      };
      self.cache.store(
        module_identifier.as_str(),
        None,
        CacheValue::new(cached_result),
      )?;
    }
    Ok(())
  }
}
