use std::sync::Arc;

use rspack_cacheable::{cacheable, with::{As, AsConverter}};
use rspack_collections::{Identifiable, IdentifierDashMap};
use rspack_error::{Result, ToStringResultToRspackResultExt};

use crate::{
  BoxModule, BuildModuleGraphArtifact, FileSystemInfo, ModuleGraph, ModuleIdentifier,
  ModuleRef, NormalModuleState, ValueCacheVersions,
  new_cache::{CacheFacade, CacheValue},
};

/// Cache for completed normal module builds.
///
/// Memory entries retain the built module by Arc. Persistent entries serialize
/// its build state, including dependency and block objects.
#[derive(Debug, Clone)]
pub(crate) struct ModuleBuildCache {
  cache: CacheFacade,
  pending: Arc<IdentifierDashMap<u64>>,
}

#[cacheable]
#[derive(Debug)]
pub(crate) struct ModuleBuildCacheEntry {
  #[cacheable(with=As<NormalModuleState>)]
  pub(crate) module_state: CachedModuleState,
}

#[derive(Debug)]
pub(crate) enum CachedModuleState {
  Shared(ModuleRef),
  Restored(Box<NormalModuleState>),
}

impl CachedModuleState {
  pub(crate) fn get(&self) -> &NormalModuleState {
    match self {
      Self::Shared(module) => module.as_normal_module().expect("only normal modules are cached").module_state(),
      Self::Restored(state) => state,
    }
  }
}

impl AsConverter<CachedModuleState> for NormalModuleState {
  fn serialize(data: &CachedModuleState, _guard: &rspack_cacheable::ContextGuard) -> rspack_cacheable::Result<Self> {
    Ok(data.get().clone())
  }
  fn deserialize(self, _guard: &rspack_cacheable::ContextGuard) -> rspack_cacheable::Result<CachedModuleState> {
    Ok(CachedModuleState::Restored(Box::new(self)))
  }
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
    module: &BoxModule,
    file_system_info: &FileSystemInfo,
    value_cache_versions: &ValueCacheVersions,
  ) -> Result<Option<CacheValue<ModuleBuildCacheEntry>>> {
    if module.as_normal_module().is_none() {
      return Ok(None);
    }

    let identifier = module.identifier();
    let Some(result) = self
      .cache
      .get::<ModuleBuildCacheEntry>(identifier.as_str(), None)
    else {
      return Ok(None);
    };
    if result
      .module_state.get()
      .need_build_with_context(file_system_info, value_cache_versions)
      .await?
    {
      return Ok(None);
    }

    Ok(Some(result))
  }

  /// Stores modules built during this phase from the final module graph.
  ///
  /// Snapshot creation and cache-entry construction are parallel. Modules,
  /// dependencies and blocks retain shared identity.
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

    let module_identifiers = snapshots
      .into_iter()
      .flatten()
      .filter_map(|(module_identifier, snapshot)| {
        let module = artifact
          .get_module_graph_mut()
          .module_by_identifier_mut(&module_identifier)?;
        module.build_info_mut().snapshot = snapshot;
        Some(module_identifier)
      })
      .collect::<Vec<_>>();

    let module_graph = artifact.get_module_graph();
    let cache_entries = rspack_parallel::scope::<_, Result<_>>(|token| {
      for module_identifier in module_identifiers {
        // SAFETY: the scope is awaited before the cache entries are published.
        let task = unsafe { token.used(module_graph) };
        task.spawn(move |module_graph| async move {
          Ok((
            module_identifier,
            create_cache_entry(module_graph, module_identifier),
          ))
        });
      }
    })
    .await
    .into_iter()
    .map(|result| result.to_rspack_result().and_then(|result| result))
    .collect::<Result<Vec<_>>>()?;

    for (module_identifier, entry) in cache_entries {
      self
        .cache
        .store(module_identifier.as_str(), None, CacheValue::new(entry));
    }
    Ok(())
  }
}

fn create_cache_entry(
  module_graph: &ModuleGraph,
  module_identifier: ModuleIdentifier,
) -> ModuleBuildCacheEntry {
  let source_module = module_graph
    .module_by_identifier(&module_identifier)
    .expect("pending module should exist in the final module graph");
  ModuleBuildCacheEntry { module_state: CachedModuleState::Shared(source_module.clone()) }
}
