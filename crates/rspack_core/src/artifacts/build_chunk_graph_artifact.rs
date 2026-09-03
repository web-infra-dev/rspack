use std::{mem, sync::Arc};

use futures::Future;
use rspack_collections::IdentifierMap;
use rspack_error::Result;
use rspack_util::{fx_hash::FxIndexMap, tracing_preset::TRACING_BENCH_TARGET};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use tracing::instrument;

use crate::{
  ArtifactExt, AsyncDependenciesBlockIdentifier, ChunkByUkey, ChunkGraph, ChunkGroupByUkey,
  ChunkGroupUkey, ChunkUkey, Compilation, ConnectionState, DependenciesBlock, DependencyType,
  Logger, ModuleIdentifier, RuntimeSpec,
  build_chunk_graph::code_splitter::CodeSplitter,
  fast_set,
  incremental::{IncrementalPasses, Mutation},
};

#[derive(Debug, Default)]
struct DependencyConditionSnapshot {
  runtimes: Vec<Option<Arc<RuntimeSpec>>>,
  async_states: HashMap<(AsyncDependenciesBlockIdentifier, usize), Vec<ConnectionState>>,
  eager_states: HashMap<(ModuleIdentifier, usize), Vec<ConnectionState>>,
}

#[derive(Debug, Default)]
pub struct BuildChunkGraphArtifact {
  pub chunk_by_ukey: ChunkByUkey,
  pub chunk_graph: ChunkGraph,
  pub chunk_group_by_ukey: ChunkGroupByUkey,
  pub entrypoints: FxIndexMap<String, ChunkGroupUkey>,
  pub async_entrypoints: Vec<ChunkGroupUkey>,
  pub named_chunk_groups: HashMap<String, ChunkGroupUkey>,
  pub named_chunks: HashMap<String, ChunkUkey>,
  pub(crate) code_splitter: CodeSplitter,
  pub module_idx: IdentifierMap<(u32, u32)>,
  dependency_condition_snapshot: DependencyConditionSnapshot,
}

impl BuildChunkGraphArtifact {
  pub(crate) fn set_code_splitter(&mut self, code_splitter: CodeSplitter) {
    fast_set(&mut self.code_splitter, code_splitter);
  }

  // we can skip rebuilding chunk graph if none of modules
  // has changed its outgoings
  // we don't need to check if module has changed its incomings
  // if it changes, the incoming module changes its outgoings as well
  fn can_skip_rebuilding(&self, this_compilation: &Compilation) -> bool {
    self.can_skip_rebuilding_legacy(this_compilation)
  }

  fn can_skip_rebuilding_legacy(&self, this_compilation: &Compilation) -> bool {
    let logger = this_compilation.get_logger("rspack.Compilation.codeSplittingCache");

    if !this_compilation.entries.keys().eq(
      this_compilation
        .build_chunk_graph_artifact
        .entrypoints
        .keys(),
    ) {
      logger.log("entrypoints change detected, rebuilding chunk graph");
      return false;
    }

    let Some(mutations) = this_compilation
      .incremental
      .mutations_read(IncrementalPasses::BUILD_MODULE_GRAPH)
    else {
      logger.log("incremental for build module graph disabled, rebuilding chunk graph");
      // if disable incremental for build module graph phase, we can't skip rebuilding
      return false;
    };

    // if we have module removal, we can't skip rebuilding
    if mutations
      .iter()
      .any(|mutation| matches!(mutation, Mutation::ModuleRemove { .. }))
    {
      logger.log("module removal detected, rebuilding chunk graph");
      return false;
    }

    let module_graph = this_compilation.get_module_graph();
    let affected_modules = mutations.get_affected_modules_with_module_graph(module_graph);
    let previous_modules_map = &this_compilation
      .build_chunk_graph_artifact
      .code_splitter
      .block_modules_runtime_map;

    if previous_modules_map.is_empty() {
      logger.log("no cache detected, rebuilding chunk graph");
      return false;
    }

    let current_condition_states = collect_async_dependency_condition_states(
      this_compilation,
      &self.dependency_condition_snapshot.runtimes,
    );
    if current_condition_states != self.dependency_condition_snapshot.async_states {
      logger.log("async dependency condition change detected, rebuilding chunk graph");
      return false;
    }

    let mut eager_modules = self
      .dependency_condition_snapshot
      .eager_states
      .keys()
      .map(|(module, _)| *module)
      .collect::<HashSet<_>>();
    eager_modules.extend(affected_modules.iter().copied());
    let current_eager_condition_states = collect_eager_dependency_condition_states(
      this_compilation,
      &self.dependency_condition_snapshot.runtimes,
      eager_modules,
    );
    if current_eager_condition_states != self.dependency_condition_snapshot.eager_states {
      logger.log("eager dependency condition change detected, rebuilding chunk graph");
      return false;
    }

    for module in affected_modules {
      if !self
        .code_splitter
        .can_reuse_affected_module(module, this_compilation)
      {
        logger.log(format!("module topology change detected: {module}"));
        return false;
      }
    }

    true
  }

  /// Reset cached chunks back to the initial render state.
  ///
  /// webpack creates fresh `Chunk` instances for every compilation, and
  /// `Chunk.rendered` starts as `false` in the constructor. Rspack can reuse
  /// cached chunks across incremental compilations, so we need to restore the
  /// same state before running the next sealing/rendering pipeline.
  fn reset_chunk_rendered_state(&mut self) {
    for chunk in self.chunk_by_ukey.values_mut() {
      chunk.set_rendered(false);
    }
  }

  fn reset_for_rebuild(&mut self) {
    self.chunk_by_ukey = Default::default();
    self.chunk_graph = Default::default();
    self.chunk_group_by_ukey = Default::default();
    self.entrypoints.clear();
    self.async_entrypoints.clear();
    self.named_chunk_groups.clear();
    self.named_chunks.clear();
    self.set_code_splitter(Default::default());
    self.module_idx.clear();
    self.dependency_condition_snapshot = Default::default();
  }
}

fn collect_async_dependency_condition_states(
  compilation: &Compilation,
  runtimes: &[Option<Arc<RuntimeSpec>>],
) -> HashMap<(AsyncDependenciesBlockIdentifier, usize), Vec<ConnectionState>> {
  let module_graph = compilation.get_module_graph();
  let module_graph_cache = &compilation.module_graph_cache_artifact;
  let side_effects_state_artifact = &compilation
    .build_module_graph_artifact
    .side_effects_state_artifact;
  let exports_info_artifact = &compilation.exports_info_artifact;
  let mut states = HashMap::default();

  for (block_id, block) in module_graph.blocks() {
    for (dependency_index, dependency_id) in block.get_dependencies().iter().enumerate() {
      let dependency = module_graph.dependency_by_id(dependency_id);
      let has_condition = dependency
        .as_module_dependency()
        .and_then(|dependency| dependency.get_condition())
        .is_some();
      if !has_condition {
        continue;
      }

      let Some(connection) = module_graph.connection_by_dependency_id(dependency_id) else {
        continue;
      };
      let dependency_states = runtimes
        .iter()
        .map(|runtime| {
          connection.active_state(
            module_graph,
            runtime.as_deref(),
            module_graph_cache,
            side_effects_state_artifact,
            exports_info_artifact,
          )
        })
        .collect();
      states.insert((*block_id, dependency_index), dependency_states);
    }
  }

  states
}

fn collect_eager_dependency_condition_states(
  compilation: &Compilation,
  runtimes: &[Option<Arc<RuntimeSpec>>],
  modules: impl IntoIterator<Item = ModuleIdentifier>,
) -> HashMap<(ModuleIdentifier, usize), Vec<ConnectionState>> {
  let module_graph = compilation.get_module_graph();
  let module_graph_cache = &compilation.module_graph_cache_artifact;
  let side_effects_state_artifact = &compilation
    .build_module_graph_artifact
    .side_effects_state_artifact;
  let exports_info_artifact = &compilation.exports_info_artifact;
  let mut states = HashMap::default();

  for module_identifier in modules {
    let Some(module) = module_graph.module_by_identifier(&module_identifier) else {
      continue;
    };
    for (dependency_index, dependency_id) in module.get_dependencies().iter().enumerate() {
      let dependency = module_graph.dependency_by_id(dependency_id);
      if !matches!(
        dependency.dependency_type(),
        DependencyType::DynamicImportEager
      ) || dependency
        .as_module_dependency()
        .and_then(|dependency| dependency.get_condition())
        .is_none()
      {
        continue;
      }

      let Some(connection) = module_graph.connection_by_dependency_id(dependency_id) else {
        continue;
      };
      let dependency_states = runtimes
        .iter()
        .map(|runtime| {
          connection.active_state(
            module_graph,
            runtime.as_deref(),
            module_graph_cache,
            side_effects_state_artifact,
            exports_info_artifact,
          )
        })
        .collect();
      states.insert((module_identifier, dependency_index), dependency_states);
    }
  }

  states
}

fn create_dependency_condition_snapshot(compilation: &Compilation) -> DependencyConditionSnapshot {
  let code_splitter = &compilation.build_chunk_graph_artifact.code_splitter;
  let mut runtimes = Vec::new();
  let mut seen_runtimes = HashSet::default();

  for runtime in code_splitter
    .block_modules_runtime_map
    .keys()
    .cloned()
    .chain(
      code_splitter
        .chunk_group_infos
        .values()
        .map(|info| Some(info.runtime.clone())),
    )
  {
    if seen_runtimes.insert(runtime.clone()) {
      runtimes.push(runtime);
    }
  }

  // A conditional async dependency implies that code splitting evaluated at
  // least one runtime. Keep a conservative fallback for an empty graph cache.
  if runtimes.is_empty() {
    runtimes.push(None);
  }

  let async_states = collect_async_dependency_condition_states(compilation, &runtimes);
  let eager_states = collect_eager_dependency_condition_states(
    compilation,
    &runtimes,
    compilation.get_module_graph().modules_keys().copied(),
  );
  DependencyConditionSnapshot {
    runtimes,
    async_states,
    eager_states,
  }
}

#[instrument(name = "Compilation:code_splitting",target=TRACING_BENCH_TARGET, skip_all)]
pub(crate) async fn use_code_splitting_cache<'a, T, F>(
  compilation: &'a mut Compilation,
  task: T,
) -> Result<()>
where
  T: Fn(&'a mut Compilation) -> F,
  F: Future<Output = Result<&'a mut Compilation>>,
{
  compilation
    .build_chunk_graph_artifact
    .reset_chunk_rendered_state();

  if !compilation.incremental.enabled() {
    task(compilation).await?;
    return Ok(());
  }

  let incremental_code_splitting = compilation
    .incremental
    .passes_enabled(IncrementalPasses::BUILD_CHUNK_GRAPH);
  let no_change = incremental_code_splitting
    && compilation
      .build_chunk_graph_artifact
      .can_skip_rebuilding(compilation);

  if no_change {
    let module_idx = &compilation.build_chunk_graph_artifact.module_idx;
    let module_graph = compilation
      .build_module_graph_artifact
      .get_module_graph_mut();
    for (m, (pre, post)) in module_idx.iter() {
      let mgm = module_graph.module_graph_module_by_identifier_mut(m);
      mgm.pre_order_index = Some(*pre);
      mgm.post_order_index = Some(*post);
    }

    return Ok(());
  }

  // Incremental chunk graph reuse did not apply, so clear the recovered
  // artifact to avoid stale data.
  compilation.build_chunk_graph_artifact.reset_for_rebuild();

  let compilation = task(compilation).await?;
  let mg = compilation.get_module_graph();
  let mut map = IdentifierMap::default();
  for (mid, mgm) in mg.module_graph_modules() {
    let (Some(pre), Some(post)) = (mgm.pre_order_index, mgm.post_order_index) else {
      continue;
    };

    map.insert(*mid, (pre, post));
  }
  compilation.build_chunk_graph_artifact.module_idx = map;
  let condition_snapshot = create_dependency_condition_snapshot(compilation);
  compilation
    .build_chunk_graph_artifact
    .dependency_condition_snapshot = condition_snapshot;
  Ok(())
}

impl ArtifactExt for BuildChunkGraphArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::BUILD_CHUNK_GRAPH;
  fn should_recover(incremental: &crate::incremental::Incremental) -> bool {
    incremental.passes_enabled(IncrementalPasses::BUILD_CHUNK_GRAPH)
  }
  fn recover(_incremental: &crate::incremental::Incremental, new: &mut Self, old: &mut Self) {
    new.code_splitter = mem::take(&mut old.code_splitter);
    new.dependency_condition_snapshot = mem::take(&mut old.dependency_condition_snapshot);
    rayon::scope(|s| {
      s.spawn(|_| new.chunk_by_ukey.clone_from(&old.chunk_by_ukey));
      s.spawn(|_| new.chunk_graph.clone_from(&old.chunk_graph));
      s.spawn(|_| new.chunk_group_by_ukey.clone_from(&old.chunk_group_by_ukey));

      s.spawn(|_| new.async_entrypoints.clone_from(&old.async_entrypoints));
      s.spawn(|_| new.named_chunk_groups.clone_from(&old.named_chunk_groups));
      s.spawn(|_| new.named_chunks.clone_from(&old.named_chunks));
      s.spawn(|_| {
        new.entrypoints.clone_from(&old.entrypoints);
        new.module_idx.clone_from(&old.module_idx);
      });
    });
  }
}
