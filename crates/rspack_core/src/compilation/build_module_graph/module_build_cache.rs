use std::sync::Arc;

use rspack_cacheable::{cacheable, utils::OwnedOrRef};
use rspack_collections::{Identifiable, IdentifierDashMap};
use rspack_error::{Result, ToStringResultToRspackResultExt};

use crate::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BoxModule,
  BuildModuleGraphArtifact, BuildResult, CacheOptions, CompilerOptions, DependenciesBlock,
  FileSystemInfo, ModuleGraph, ModuleIdentifier, NormalModuleState, OptimizationBailoutItem,
  ValueCacheVersions,
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
  codec: CacheCodec,
  pending: Arc<IdentifierDashMap<u64>>,
}

#[cacheable]
#[derive(Debug)]
pub(crate) struct ModuleBuildCacheEntry<'a> {
  module_state: OwnedOrRef<'a, NormalModuleState>,
  dependencies: Vec<BoxDependency>,
  // Preserve BuildResult's ownership shape without reallocating large nested blocks on restore.
  #[allow(clippy::vec_box)]
  blocks: Vec<Box<AsyncDependenciesBlock>>,
  optimization_bailouts: Vec<OptimizationBailoutItem>,
}

impl ModuleBuildCacheEntry<'_> {
  pub(crate) fn into_build_result(self, mut module: BoxModule) -> BuildResult {
    module
      .as_normal_module_mut()
      .expect("module cache entries are only restored for normal modules")
      .restore_module_state(self.module_state.into_owned());
    BuildResult {
      module,
      dependencies: self.dependencies,
      blocks: self.blocks,
      optimization_bailouts: self.optimization_bailouts,
    }
  }
}

impl ModuleBuildCache {
  pub(crate) fn new(cache: CacheFacade, options: &CompilerOptions) -> Self {
    let project_root = match &options.cache {
      CacheOptions::Persistent(cache_options) if cache_options.portable => {
        Some(options.context.as_path().to_path_buf())
      }
      _ => None,
    };
    Self {
      cache,
      codec: CacheCodec::new(project_root),
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
  ) -> Result<Option<ModuleBuildCacheEntry<'static>>> {
    if module.as_normal_module().is_none() {
      return Ok(None);
    }

    let identifier = module.identifier();
    let Some(bytes) = self.cache.get::<Vec<u8>>(identifier.as_str(), None)? else {
      return Ok(None);
    };
    let result = self
      .codec
      .decode::<ModuleBuildCacheEntry<'static>>(&bytes)?;
    if result
      .module_state
      .as_ref()
      .need_build_with_context(file_system_info, value_cache_versions)
      .await?
    {
      return Ok(None);
    }

    Ok(Some(result))
  }

  /// Stores modules built during this phase from the final module graph.
  ///
  /// Snapshot creation and encoding are parallel. The codec reconstructs
  /// independently owned module state, dependencies, and blocks because the
  /// final module graph owns the values being cached.
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
    let encoded_results = rspack_parallel::scope::<_, Result<_>>(|token| {
      for module_identifier in module_identifiers {
        // SAFETY: the scope is awaited before the cache entries are published.
        let task = unsafe { token.used((module_graph, &self.codec)) };
        task.spawn(move |(module_graph, codec)| async move {
          Ok((
            module_identifier,
            encode_cache_entry(module_graph, module_identifier, codec),
          ))
        });
      }
    })
    .await
    .into_iter()
    .map(|result| result.to_rspack_result().and_then(|result| result))
    .collect::<Result<Vec<_>>>()?;

    for (module_identifier, bytes) in encoded_results {
      let bytes = match bytes {
        Ok(Some(bytes)) => bytes,
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
        .store(module_identifier.as_str(), None, CacheValue::new(bytes))?;
    }
    Ok(())
  }
}

fn encode_cache_entry(
  module_graph: &ModuleGraph,
  module_identifier: ModuleIdentifier,
  codec: &CacheCodec,
) -> Result<Option<Vec<u8>>> {
  let source_module = module_graph
    .module_by_identifier(&module_identifier)
    .expect("pending module should exist in the final module graph");
  let normal_module = source_module
    .as_normal_module()
    .expect("only normal modules are marked pending for the module build cache");
  let Some(dependencies) =
    clone_dependencies(module_graph, source_module.get_dependencies(), codec)?
  else {
    return Ok(None);
  };
  let blocks = source_module
    .get_blocks()
    .iter()
    .map(|block_id| clone_block(module_graph, block_id, codec))
    .collect::<Result<Option<Vec<_>>>>()?;
  let Some(blocks) = blocks else {
    return Ok(None);
  };
  let optimization_bailouts = module_graph
    .get_optimization_bailout(&module_identifier)
    .clone();

  codec
    .encode(&ModuleBuildCacheEntry {
      module_state: normal_module.module_state().into(),
      dependencies,
      blocks,
      optimization_bailouts,
    })
    .map(Some)
}

fn clone_dependencies(
  module_graph: &ModuleGraph,
  dependency_ids: &[crate::DependencyId],
  codec: &CacheCodec,
) -> Result<Option<Vec<BoxDependency>>> {
  let Some(dependencies) = dependency_ids
    .iter()
    .map(|dependency_id| {
      crate::module_graph::internal::try_dependency_ref_by_id(module_graph, dependency_id)
    })
    .collect::<Option<Vec<_>>>()
  else {
    return Ok(None);
  };
  // `DependencyRef` and `BoxDependency` intentionally share the same archived
  // representation, so decoding turns graph-owned references back into the
  // uniquely owned dependencies required by `BuildResult`.
  codec.decode(&codec.encode(&dependencies)?).map(Some)
}

fn clone_block(
  module_graph: &ModuleGraph,
  block_id: &AsyncDependenciesBlockIdentifier,
  codec: &CacheCodec,
) -> Result<Option<Box<AsyncDependenciesBlock>>> {
  let Some(source) = module_graph.block_by_id(block_id) else {
    return Ok(None);
  };
  let mut block = codec.decode::<AsyncDependenciesBlock>(&codec.encode(source)?)?;
  let Some(dependencies) = clone_dependencies(module_graph, source.get_dependencies(), codec)?
  else {
    return Ok(None);
  };
  let blocks = source
    .get_blocks()
    .iter()
    .map(|block_id| clone_block(module_graph, block_id, codec))
    .collect::<Result<Option<Vec<_>>>>()?;
  let Some(blocks) = blocks else {
    return Ok(None);
  };
  block.restore_build_result(dependencies, blocks);
  Ok(Some(Box::new(block)))
}
