use std::sync::Arc;

use rspack_collections::{Identifiable, IdentifierDashMap};
use rspack_error::{Result, ToStringResultToRspackResultExt};

use crate::{
  BoxModule, BuildModuleGraphArtifact, FileSystemInfo, ModuleGraph, ModuleIdentifier, ModuleRef,
  NeedBuildContext, ValueCacheVersions,
  new_cache::{CacheFacade, CacheValue, Etag},
};

/// Cache for completed module builds, validated by each module's `need_build` implementation.
///
/// Memory entries share the built module with the graph. Persistent entries
/// encode the same module directly, including its dependencies and build metadata.
#[derive(Debug, Clone)]
pub(crate) struct ModuleBuildCache {
  cache: CacheFacade,
  restore_enabled: bool,
  pending: Arc<IdentifierDashMap<(u64, Option<Etag>)>>,
}

impl ModuleBuildCache {
  pub(crate) fn new(cache: CacheFacade, restore_enabled: bool) -> Self {
    Self {
      cache,
      restore_enabled,
      pending: Default::default(),
    }
  }

  /// Defers publishing a built module until the build-module-graph phase has
  /// completed, so make-stage mutations are included in the cache entry.
  pub(crate) fn mark_pending(
    &self,
    module_identifier: ModuleIdentifier,
    build_start_time: u64,
    etag: Option<Etag>,
  ) {
    self
      .pending
      .insert(module_identifier, (build_start_time, etag));
  }

  pub(crate) fn etag(&self, module: &BoxModule) -> Option<Etag> {
    let profile = self.cache.module_profile();
    let loader_profile = self.cache.loader_profile();
    if profile.is_none() && loader_profile.is_none() {
      return None;
    }
    module
      .as_normal_module()?
      .reuse_etag(profile, loader_profile)
  }

  pub(crate) async fn restore(
    &self,
    module: &BoxModule,
    file_system_info: &FileSystemInfo,
    value_cache_versions: &ValueCacheVersions,
  ) -> Result<Option<ModuleRef>> {
    if !self.restore_enabled {
      return Ok(None);
    }

    let identifier = module.identifier();
    let etag = self.etag(module);
    let context = NeedBuildContext::new(file_system_info, value_cache_versions);
    if let Some(result) = self
      .cache
      .get::<ModuleRef>(identifier.as_str(), etag.clone())
      && !result.need_build(&context).await?
    {
      result.reset_for_compilation(module.factory_meta());
      return Ok(Some(result.as_arc().as_ref().clone()));
    }
    if self.cache.module_profile().is_some()
      && let Some(etag) = etag
      && let Some(result) = self
        .cache
        .get_imported::<ModuleRef>(identifier.as_str(), etag.clone())
      && !result.need_build(&context).await?
    {
      result.reset_for_compilation(module.factory_meta());
      self
        .cache
        .store(identifier.as_str(), Some(etag), result.clone());
      return Ok(Some(result.as_arc().as_ref().clone()));
    }
    Ok(None)
  }

  /// Stores modules built during this phase from the final module graph.
  ///
  /// Cache preparation and cache-entry construction are parallel. Modules,
  /// dependencies and blocks retain shared identity.
  pub(crate) async fn store_pending(
    &self,
    artifact: &mut BuildModuleGraphArtifact,
    file_system_info: &FileSystemInfo,
  ) -> Result<()> {
    let pending = self
      .pending
      .iter()
      .map(|entry| (*entry.key(), entry.value().clone()))
      .collect::<Vec<_>>();
    self.pending.clear();

    let module_graph = artifact.get_module_graph();
    let prepared_modules = rspack_parallel::scope::<_, Result<_>>(|token| {
      for (module_identifier, (build_start_time, etag)) in pending {
        // SAFETY: the scope is awaited before the module graph is mutated.
        let task = unsafe { token.used((module_graph, file_system_info)) };
        task.spawn(move |(module_graph, file_system_info)| async move {
          let Some(module) = module_graph.module_by_identifier(&module_identifier) else {
            return Ok(None);
          };
          module
            .prepare_for_cache(file_system_info, build_start_time)
            .await?;
          Ok(Some((module_identifier, etag)))
        });
      }
    })
    .await
    .into_iter()
    .map(|result| result.to_rspack_result().and_then(|result| result))
    .collect::<Result<Vec<_>>>()?;

    let module_identifiers = prepared_modules
      .into_iter()
      .flatten()
      .filter_map(|(module_identifier, etag)| {
        let module = artifact
          .get_module_graph()
          .module_by_identifier(&module_identifier)?;
        module.freeze_build_info();
        Some((module_identifier, etag))
      })
      .collect::<Vec<_>>();

    let module_graph = artifact.get_module_graph();
    let cache_entries = rspack_parallel::scope::<_, Result<_>>(|token| {
      for (module_identifier, etag) in module_identifiers {
        // SAFETY: the scope is awaited before the cache entries are published.
        let task = unsafe { token.used(module_graph) };
        task.spawn(move |module_graph| async move {
          Ok((
            module_identifier,
            etag,
            create_cache_entry(module_graph, module_identifier),
          ))
        });
      }
    })
    .await
    .into_iter()
    .map(|result| result.to_rspack_result().and_then(|result| result))
    .collect::<Result<Vec<_>>>()?;

    for (module_identifier, etag, entry) in cache_entries {
      self
        .cache
        .store(module_identifier.as_str(), etag, CacheValue::new(entry));
    }
    Ok(())
  }
}

fn create_cache_entry(
  module_graph: &ModuleGraph,
  module_identifier: ModuleIdentifier,
) -> ModuleRef {
  let source_module = module_graph
    .module_by_identifier(&module_identifier)
    .expect("pending module should exist in the final module graph");
  source_module.clone()
}
