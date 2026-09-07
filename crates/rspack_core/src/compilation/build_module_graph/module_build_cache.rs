use std::sync::Arc;

use rspack_cacheable::cacheable;
use rspack_collections::{Identifiable, IdentifierDashMap};
use rspack_error::{Result, ToStringResultToRspackResultExt};

use crate::{
  AsyncDependenciesBlockBuildResult, AsyncDependenciesBlockIdentifier, BoxModule,
  BuildModuleGraphArtifact, BuildResult, CacheOptions, CompilerOptions, DependenciesBlock,
  DependencyRef, FileSystemInfo, ModuleGraph, ModuleIdentifier, ModuleState, NeedBuildContext,
  OptimizationBailoutItem, ValueCacheVersions,
  cache::{CacheCodec, SnapshotStrategyOptions},
  new_cache::{CacheFacade, CacheValue},
};

/// Cache for completed module builds.
///
/// Cache entries store only [`ModuleState`] plus the graph-owned build
/// output. Factory-owned module data is always supplied by the fresh module.
#[derive(Debug, Clone)]
pub(crate) struct ModuleBuildCache {
  cache: CacheFacade,
  persistent_codec: Option<CacheCodec>,
  pending: Arc<IdentifierDashMap<u64>>,
}

#[cacheable]
#[derive(Debug)]
pub(crate) struct ModuleBuildCacheEntry {
  module_type: String,
  module_state: Box<dyn ModuleState>,
  graph_result: ModuleGraphBuildResult,
}

#[cacheable]
#[derive(Debug, Clone)]
struct ModuleGraphBuildResult {
  dependencies: Vec<DependencyRef>,
  blocks: Vec<AsyncDependenciesBlockBuildResult>,
  optimization_bailouts: Vec<OptimizationBailoutItem>,
}

impl ModuleBuildCacheEntry {
  pub(crate) fn to_build_result(&self, module: BoxModule) -> BuildResult {
    BuildResult {
      module,
      dependencies: self.graph_result.dependencies.clone(),
      blocks: self.graph_result.blocks.clone(),
      optimization_bailouts: self.graph_result.optimization_bailouts.clone(),
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
    module: &mut BoxModule,
    file_system_info: &FileSystemInfo,
    value_cache_versions: &ValueCacheVersions,
  ) -> Result<Option<CacheValue<ModuleBuildCacheEntry>>> {
    // Factory data can explicitly disallow reuse, for example an extracted CSS
    // dependency produced by a non-cacheable loader in the current compilation.
    if !module.build_info().cacheable {
      return Ok(None);
    }

    let identifier = module.identifier();
    let Some(result) = self
      .cache
      .get::<ModuleBuildCacheEntry>(identifier.as_str(), None)?
    else {
      return Ok(None);
    };
    if result.module_type != module.build_cache_type() {
      return Ok(None);
    }
    let Some(fresh_state) = module.restore_build_state(result.module_state.as_ref()) else {
      return Ok(None);
    };
    if !module.build_info().cacheable
      || module
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.is_error())
      || value_cache_versions.has_diff(&module.build_info().value_dependencies)
    {
      module.restore_build_state(fresh_state.as_ref());
      return Ok(None);
    }
    let need_build = module
      .need_build(&NeedBuildContext::new(
        file_system_info,
        value_cache_versions,
      ))
      .await?;
    if need_build {
      module.restore_build_state(fresh_state.as_ref());
      return Ok(None);
    }

    Ok(Some(result))
  }

  /// Stores modules built during this phase from the final module graph.
  ///
  /// Snapshot creation and cache-entry construction are parallel. Module state
  /// and graph containers are cloned, while blocks and dependencies retain shared identity.
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
          let build_info = module.build_info();
          let snapshot = if build_info.cacheable
            && !module
              .diagnostics()
              .iter()
              .any(|diagnostic| diagnostic.is_error())
          {
            Some(
              file_system_info
                .create_snapshot(
                  Some(build_start_time),
                  &build_info.dependencies.file,
                  &build_info.dependencies.context,
                  &build_info.dependencies.missing,
                  SnapshotStrategyOptions::timestamp(),
                )
                .await?,
            )
          } else {
            None
          };
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
  let Some(module_state) = source_module.capture_build_state() else {
    return Ok(None);
  };
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
    module_type: source_module.build_cache_type().to_owned(),
    module_state,
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
