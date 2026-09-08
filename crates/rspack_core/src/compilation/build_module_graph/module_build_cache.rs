use std::sync::Arc;

use rspack_collections::{Identifiable, IdentifierDashMap};
use rspack_error::{Result, ToStringResultToRspackResultExt};

use crate::{
  BoxModule, BuildModuleGraphArtifact, FileSystemInfo, ModuleGraph, ModuleIdentifier, ModuleRef,
  NeedBuildContext, ValueCacheVersions, new_cache::CacheFacade,
};

/// Cache ownership of live modules. Graphs/build tasks hold the exclusive access
/// lease; the cache holds only the shared cell. Persistent encoding borrows the
/// graph at the end of compilation and never runs on a live module in idle.
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
    fresh: BoxModule,
    file_system_info: &FileSystemInfo,
    value_cache_versions: &ValueCacheVersions,
  ) -> Result<(BoxModule, bool)> {
    if !fresh.supports_module_cache() {
      return Ok((fresh, false));
    }
    let identifier = fresh.identifier();
    let mut fresh = Some(fresh);
    let mut decoded_module = None;
    let value = self.cache.get_live(identifier.as_str(), |bytes, codec| {
      // Disk decoding uses the freshly factorized module to supply runtime-only
      // fields. Once restored, the memory layer owns a complete ModuleCell.
      fresh
        .as_mut()
        .expect("fresh module should exist")
        .restore_from_cache(bytes, codec)?;
      let mut module = fresh.take().expect("fresh module should exist");
      let reference = module.share();
      module.set_cache_valid(true);
      decoded_module = Some(module);
      Ok(reference.cache_value())
    });
    let Some(value) = value else {
      return Ok((
        fresh.expect("a cache miss must retain the fresh module"),
        false,
      ));
    };
    let Some(mut module) = decoded_module.or_else(|| ModuleRef::from_cache(value).try_acquire())
    else {
      // Another graph (including rollback history or the module executor) may
      // still own this version. Do not alias its lease or fall back to disk.
      return Ok((
        fresh.expect("an occupied cache entry must retain the fresh module"),
        false,
      ));
    };
    if let Some(mut fresh) = fresh {
      if module.as_ref().as_any().type_id() != fresh.as_ref().as_any().type_id() {
        return Ok((fresh, false));
      }
      module.update_cache_module(fresh.as_mut());
    }
    let valid = module.cache_valid();
    // Validation may itself mutate a module or fail asynchronously. Until it
    // completes, another attempt must treat this version as invalid.
    module.set_cache_valid(false);
    let reused = valid
      && !module
        .need_build(&NeedBuildContext::new(
          file_system_info,
          value_cache_versions,
        ))
        .await?;
    module.set_cache_valid(reused);
    if !reused {
      module.reset_build_state();
    }
    Ok((module, reused))
  }

  pub(crate) fn track(&self, module: &mut BoxModule, built: bool) {
    if !module.supports_module_cache() {
      return;
    }
    let reference = module.share();
    if built {
      module.set_cache_valid(false);
    }
    self
      .cache
      .store_live(module.identifier().as_str(), reference.cache_value());
  }

  /// Complete validity metadata without copying module state or dependencies.
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
      let Some(module) = artifact
        .get_module_graph_mut()
        .module_by_identifier_mut(&module_identifier)
      else {
        continue;
      };
      let valid = snapshot.is_some();
      module.build_info_mut().snapshot = snapshot;
      module.set_cache_valid(valid);
      if module.supports_module_cache() {
        self
          .cache
          .store_live(module_identifier.as_str(), module.share().cache_value());
      }
    }
    Ok(())
  }

  pub(crate) fn encode(&self, module_graph: &mut ModuleGraph) {
    if !self.cache.has_file_cache() {
      return;
    }
    let identifiers = module_graph
      .modules()
      .filter_map(|(id, module)| module.supports_module_cache().then_some(*id))
      .collect::<Vec<_>>();
    for identifier in identifiers {
      let module = module_graph
        .module_by_identifier_mut(&identifier)
        .expect("module should exist");
      let reference = module.share();
      if !module.cache_valid() {
        self
          .cache
          .store_live(identifier.as_str(), reference.cache_value());
        continue;
      }
      self
        .cache
        .encode_live(identifier.as_str(), reference.cache_value(), |codec| {
          module.serialize_for_cache(codec)
        });
    }
  }
}
