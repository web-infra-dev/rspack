use std::sync::{Arc, Mutex};

use rayon::prelude::*;
use rspack_cacheable::{cacheable, utils::OwnedOrRef, with::AsOwned};
use rspack_collections::{Identifiable, IdentifierDashMap, IdentifierMap};
use rspack_error::{Result, ToStringResultToRspackResultExt};

use crate::{
  AsyncDependenciesBlockBuildResult, AsyncDependenciesBlockIdentifier, BoxModule,
  BuildModuleGraphArtifact, BuildResult, DependenciesBlock, DependencyRef, FileSystemInfo,
  ModuleGraph, ModuleIdentifier, NormalModuleState, ValueCacheVersions,
  new_cache::{Cache, CacheFacade, CacheValue, MemoryCacheGetResult},
};

/// Cache for completed normal module builds.
///
/// A compilation exclusively owns its live modules until the next full build
/// transfers them back to the memory cache. Filesystem writes borrow the latest
/// state at compiler boundaries and queue only encoded bytes for background I/O.
#[derive(Debug, Clone)]
pub(crate) struct ModuleBuildCache {
  cache: CacheFacade,
  pending: Arc<IdentifierDashMap<u64>>,
}

#[derive(Debug)]
struct CachedModule {
  module: BoxModule,
  graph_result: ModuleGraphBuildResult,
}

// None records an acquired module, preventing fallback to stale filesystem data.
type ModuleCacheSlot = Mutex<Option<CachedModule>>;

#[cacheable]
struct StoredModule<'a> {
  #[cacheable(with=AsOwned)]
  module_state: OwnedOrRef<'a, NormalModuleState>,
  graph_result: Option<ModuleGraphBuildResult>,
}

#[cacheable]
#[derive(Debug)]
pub(crate) struct ModuleGraphBuildResult {
  dependencies: Vec<DependencyRef>,
  blocks: Vec<AsyncDependenciesBlockBuildResult>,
}

impl ModuleGraphBuildResult {
  pub(crate) fn into_build_result(self, module: BoxModule) -> BuildResult {
    let optimization_bailouts = module
      .as_normal_module()
      .expect("only normal modules are cached")
      .build_optimization_bailouts()
      .to_vec();
    BuildResult {
      module,
      dependencies: self.dependencies,
      blocks: self.blocks,
      optimization_bailouts,
    }
  }
}

impl ModuleBuildCache {
  pub(crate) fn new(cache: &Cache) -> Self {
    Self {
      // The owned filesystem representation differs from the previous state cache.
      cache: cache.facade("Compilation/modules/owned-v1"),
      pending: Default::default(),
    }
  }

  /// Defers filesystem dependency snapshot creation until make-stage mutations finish.
  pub(crate) fn mark_pending(&self, module_identifier: ModuleIdentifier, build_start_time: u64) {
    self.pending.insert(module_identifier, build_start_time);
  }

  pub(crate) async fn restore(
    &self,
    mut module: BoxModule,
    file_system_info: &FileSystemInfo,
    value_cache_versions: &ValueCacheVersions,
  ) -> Result<(BoxModule, Option<ModuleGraphBuildResult>)> {
    if module.as_normal_module().is_none() {
      return Ok((module, None));
    }

    let identifier = module.identifier();
    let cached = match self
      .cache
      .get_memory::<ModuleCacheSlot>(identifier.as_str(), None)
    {
      MemoryCacheGetResult::Hit(slot) => slot
        .lock()
        .expect("module cache slot should not be poisoned")
        .take(),
      MemoryCacheGetResult::Miss => None,
      MemoryCacheGetResult::NotCached => {
        let stored = self
          .cache
          .restore_owned::<StoredModule<'static>>(identifier.as_str(), None);
        self.mark_acquired(identifier);
        if let Some(StoredModule {
          module_state,
          graph_result: Some(graph_result),
        }) = stored
          && !module_state
            .as_ref()
            .need_build_with_context(file_system_info, value_cache_versions)
            .await?
        {
          module
            .as_normal_module_mut()
            .expect("normal module was checked")
            .restore_module_state(module_state.into_owned());
          return Ok((module, Some(graph_result)));
        }
        return Ok((module, None));
      }
    };
    self.mark_acquired(identifier);
    let Some(mut cached) = cached else {
      return Ok((module, None));
    };
    let fresh = module
      .as_normal_module_mut()
      .expect("normal module was checked");
    let normal = cached
      .module
      .as_normal_module_mut()
      .expect("only normal modules are cached");
    normal.update_cache_module(fresh);
    if normal
      .need_build_with_context(file_system_info, value_cache_versions)
      .await?
    {
      normal.reset_cached_build(fresh);
      return Ok((cached.module, None));
    }

    Ok((cached.module, Some(cached.graph_result)))
  }

  fn mark_acquired(&self, identifier: ModuleIdentifier) {
    self.cache.store_memory(
      identifier.as_str(),
      None,
      CacheValue::new(ModuleCacheSlot::new(None)),
    );
  }

  /// Creates validity snapshots without copying the mutable module state.
  pub(crate) async fn snapshot_pending(
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
          let snapshot = module
            .create_cache_snapshot(file_system_info, build_start_time)
            .await?;
          Ok(Some((module_identifier, snapshot)))
        });
      }
    })
    .await
    .into_iter()
    .map(|result| result.to_rspack_result().and_then(|result| result))
    .collect::<Result<Vec<_>>>()?;

    for (module_identifier, snapshot) in snapshots.into_iter().flatten() {
      if let Some(module) = artifact
        .get_module_graph_mut()
        .module_by_identifier_mut(&module_identifier)
      {
        module.build_info_mut().snapshot = snapshot;
      }
    }
    Ok(())
  }

  /// Encodes current module state while the compilation is exclusively borrowed.
  /// Include cache hits too: later hooks may have changed their state.
  pub(crate) fn store_modules(&self, module_graph: &ModuleGraph) {
    if !self.cache.has_file_cache() {
      return;
    }
    self.cache.store_borrowed(
      module_graph
        .modules_par()
        .filter_map(|(identifier, module)| {
          let normal = module.as_normal_module()?;
          Some((
            identifier.as_str(),
            None,
            StoredModule {
              module_state: OwnedOrRef::Borrowed(normal.module_state()),
              graph_result: collect_graph_result(module_graph, module),
            },
          ))
        }),
    );
  }

  /// Returns live modules immediately before a full build discards their graph.
  /// Incremental rebuilds keep ownership in their recovered graph instead.
  pub(crate) fn release_modules(&self, module_graph: &mut ModuleGraph) {
    if !self.cache.has_memory_cache() {
      return;
    }
    let mut results: IdentifierMap<_> = module_graph
      .modules()
      .filter(|(_, module)| module.as_normal_module().is_some())
      .filter_map(|(identifier, module)| {
        collect_graph_result(module_graph, module).map(|result| (*identifier, result))
      })
      .collect();
    for module in module_graph.take_modules() {
      let identifier = module.identifier();
      let Some(graph_result) = results.remove(&identifier) else {
        if module.as_normal_module().is_some() {
          self.mark_acquired(identifier);
        }
        continue;
      };
      self.cache.store_memory(
        identifier.as_str(),
        None,
        CacheValue::new(ModuleCacheSlot::new(Some(CachedModule {
          module,
          graph_result,
        }))),
      );
    }
  }
}

fn collect_graph_result(
  module_graph: &ModuleGraph,
  source_module: &BoxModule,
) -> Option<ModuleGraphBuildResult> {
  let dependencies = clone_dependencies(module_graph, source_module.get_dependencies())?;
  let blocks = source_module
    .get_blocks()
    .iter()
    .map(|block_id| clone_block(module_graph, block_id))
    .collect::<Option<Vec<_>>>()?;
  Some(ModuleGraphBuildResult {
    dependencies,
    blocks,
  })
}

fn clone_dependencies(
  module_graph: &ModuleGraph,
  dependency_ids: &[crate::DependencyId],
) -> Option<Vec<DependencyRef>> {
  dependency_ids
    .iter()
    .map(|dependency_id| {
      crate::module_graph::internal::try_dependency_ref_by_id(module_graph, dependency_id)
    })
    .collect()
}

fn clone_block(
  module_graph: &ModuleGraph,
  block_id: &AsyncDependenciesBlockIdentifier,
) -> Option<AsyncDependenciesBlockBuildResult> {
  let block = module_graph.block_ref_by_id(block_id)?.clone();
  let dependencies = clone_dependencies(module_graph, block.get_dependencies())?;
  let blocks = block
    .get_blocks()
    .iter()
    .map(|block_id| clone_block(module_graph, block_id))
    .collect::<Option<Vec<_>>>()?;
  Some(AsyncDependenciesBlockBuildResult {
    block,
    dependencies,
    blocks,
  })
}
