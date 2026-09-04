use std::sync::Arc;

use rspack_cacheable::cacheable;
use rspack_collections::{Identifiable, IdentifierDashMap};
use rspack_error::{Result, ToStringResultToRspackResultExt};

use crate::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxModule, BuildModuleGraphArtifact,
  BuildResult, CacheOptions, CompilerOptions, DependenciesBlock, DependencyRef, FileSystemInfo,
  ModuleGraph, ModuleIdentifier, NormalModuleState, OptimizationBailoutItem, ValueCacheVersions,
  cache::CacheCodec,
  new_cache::{CacheFacade, CacheValue},
};

/// Cache for completed normal module builds.
///
/// Cache entries store only [`NormalModuleState`] plus the graph-owned build
/// output. Factory-owned module data is always supplied by the fresh module.
#[derive(Debug, Clone)]
pub(crate) struct ModuleBuildCache {
  cache: CacheFacade,
  persistent_codec: Option<CacheCodec>,
  pending: Arc<IdentifierDashMap<u64>>,
}

#[cacheable]
#[derive(Debug, Clone)]
pub(crate) struct ModuleBuildCacheEntry {
  module_state: NormalModuleState,
  graph_result: ModuleGraphBuildResult,
}

#[cacheable]
#[derive(Debug, Clone)]
struct ModuleGraphBuildResult {
  dependencies: Vec<DependencyRef>,
  // Preserve BuildResult's ownership shape without reallocating large nested blocks on restore.
  #[allow(clippy::vec_box)]
  blocks: Vec<Box<AsyncDependenciesBlock>>,
  optimization_bailouts: Vec<OptimizationBailoutItem>,
}

impl ModuleBuildCacheEntry {
  pub(crate) fn into_build_result(self, mut module: BoxModule) -> BuildResult {
    module
      .as_normal_module_mut()
      .expect("module cache entries are only restored for normal modules")
      .restore_module_state(self.module_state);
    BuildResult {
      module,
      dependencies: self.graph_result.dependencies,
      blocks: self.graph_result.blocks,
      optimization_bailouts: self.graph_result.optimization_bailouts,
    }
  }
}

impl ModuleBuildCache {
  pub(crate) fn new(cache: CacheFacade, options: &CompilerOptions) -> Self {
    let persistent_codec = match &options.cache {
      CacheOptions::Persistent(cache_options) if cache_options.portable => Some(CacheCodec::new(
        Some(options.context.as_path().to_path_buf()),
      )),
      CacheOptions::Persistent(_) => Some(CacheCodec::new(None)),
      _ => None,
    };
    Self {
      cache,
      persistent_codec,
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
  ) -> Result<Option<ModuleBuildCacheEntry>> {
    if module.as_normal_module().is_none() {
      return Ok(None);
    }

    let identifier = module.identifier();
    let Some(result) = self
      .cache
      .get::<ModuleBuildCacheEntry>(identifier.as_str(), None)?
    else {
      return Ok(None);
    };
    if result
      .module_state
      .need_build_with_context(file_system_info, value_cache_versions)
      .await?
    {
      return Ok(None);
    }

    Ok(Some(result.as_arc().as_ref().clone()))
  }

  /// Stores modules built during this phase from the final module graph.
  ///
  /// Snapshot creation and cache-entry construction are parallel. Module state
  /// and graph containers are cloned, while dependencies retain webpack-style
  /// shared identity through [`DependencyRef`].
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
        let task = unsafe { token.used((module_graph, self.persistent_codec.as_ref())) };
        task.spawn(move |(module_graph, codec)| async move {
          Ok((
            module_identifier,
            create_cache_entry(module_graph, module_identifier, codec),
          ))
        });
      }
    })
    .await
    .into_iter()
    .map(|result| result.to_rspack_result().and_then(|result| result))
    .collect::<Result<Vec<_>>>()?;

    for (module_identifier, entry) in cache_entries {
      let entry = match entry {
        Ok(Some(entry)) => entry,
        Ok(None) => continue,
        Err(error) => {
          // Match webpack's persistent cache behavior: an unsupported entry is
          // skipped instead of failing the compilation.
          tracing::debug!(
            module = module_identifier.as_str(),
            %error,
            "Skipped non-serializable module cache entry"
          );
          continue;
        }
      };
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
  persistent_codec: Option<&CacheCodec>,
) -> Result<Option<ModuleBuildCacheEntry>> {
  let source_module = module_graph
    .module_by_identifier(&module_identifier)
    .expect("pending module should exist in the final module graph");
  let normal_module = source_module
    .as_normal_module()
    .expect("only normal modules are marked pending for the module build cache");
  let Some(dependencies) = clone_dependencies(module_graph, source_module.get_dependencies())
  else {
    return Ok(None);
  };
  let blocks = source_module
    .get_blocks()
    .iter()
    .map(|block_id| clone_block(module_graph, block_id))
    .collect::<Option<Vec<_>>>();
  let Some(blocks) = blocks else {
    return Ok(None);
  };
  let optimization_bailouts = module_graph
    .get_optimization_bailout(&module_identifier)
    .clone();

  let entry = ModuleBuildCacheEntry {
    module_state: normal_module.module_state().clone(),
    graph_result: ModuleGraphBuildResult {
      dependencies,
      blocks,
      optimization_bailouts,
    },
  };
  if let Some(codec) = persistent_codec {
    codec.encode(&entry)?;
  }
  Ok(Some(entry))
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
) -> Option<Box<AsyncDependenciesBlock>> {
  let Some(source) = module_graph.block_by_id(block_id) else {
    return None;
  };
  let mut block = source.clone();
  let Some(dependencies) = clone_dependencies(module_graph, source.get_dependencies()) else {
    return None;
  };
  let blocks = source
    .get_blocks()
    .iter()
    .map(|block_id| clone_block(module_graph, block_id))
    .collect::<Option<Vec<_>>>();
  let Some(blocks) = blocks else {
    return None;
  };
  block.restore_build_result(dependencies, blocks);
  Some(Box::new(block))
}
