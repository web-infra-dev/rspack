use std::mem;

use futures::Future;
use rspack_collections::{IdentifierIndexMap, IdentifierMap};
use rspack_error::Result;
use rspack_util::{fx_hash::FxIndexMap, tracing_preset::TRACING_BENCH_TARGET};
use rustc_hash::FxHashMap as HashMap;
use tracing::instrument;

use crate::{
  ArtifactExt, ChunkByUkey, ChunkGraph, ChunkGroupByUkey, ChunkGroupKind, ChunkGroupUkey,
  ChunkUkey, Compilation, DependenciesBlock, GroupOptions, Logger,
  build_chunk_graph::code_splitter::{CodeSplitter, DependenciesBlockIdentifier},
  fast_set,
  incremental::{IncrementalPasses, Mutation},
};

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
    let module_graph_cache = &this_compilation.module_graph_cache_artifact;
    let affected_modules = mutations.get_affected_modules_with_module_graph(module_graph);
    let previous_modules_map = &this_compilation
      .build_chunk_graph_artifact
      .code_splitter
      .block_modules_runtime_map;

    if previous_modules_map.is_empty() {
      logger.log("no cache detected, rebuilding chunk graph");
      return false;
    }

    for module in affected_modules {
      let module_graph_module = module_graph
        .module_graph_module_by_identifier(&module)
        .expect("should have module");
      let current_blocks = module_graph
        .module_by_identifier(&module)
        .expect("should have module")
        .get_blocks();
      let previous_blocks = self
        .code_splitter
        .prepared_blocks_map
        .get(&DependenciesBlockIdentifier::Module(module))
        .map(Vec::as_slice)
        .unwrap_or_default();

      if current_blocks != previous_blocks {
        logger.log(format!("module async blocks change detected: {module}"));
        return false;
      }

      for block_id in current_blocks {
        let block = module_graph.block_by_id_expect(block_id);
        // Nested async blocks are not currently constructed, but avoid
        // reusing an incomplete topology if support is added later.
        if !block.get_blocks().is_empty() {
          logger.log(format!("nested async blocks detected: {module}"));
          return false;
        }

        let Some(previous_chunk_group) = self
          .chunk_graph
          .get_block_chunk_group(block_id, &self.chunk_group_by_ukey)
        else {
          continue;
        };
        let same_group_options = match (block.get_group_options(), &previous_chunk_group.kind) {
          (None, ChunkGroupKind::Normal { options }) => options == &Default::default(),
          (Some(GroupOptions::ChunkGroup(current)), ChunkGroupKind::Normal { options }) => {
            current == options
          }
          (Some(GroupOptions::Entrypoint(current)), ChunkGroupKind::Entrypoint { options, .. }) => {
            current == options
          }
          _ => false,
        };

        if !same_group_options {
          logger.log(format!(
            "module async block options change detected: {module}"
          ));
          return false;
        }
      }

      // Match CodeSplitter::prepare: ESM dependencies are ordered by source
      // order and unordered dependencies are appended afterwards. Keep root
      // and async-block connections separate; all_dependencies contains both.
      let mut ordered_dependencies = vec![];
      let mut unordered_dependencies = vec![];
      for dep_id in module_graph_module.all_dependencies() {
        let dependency = module_graph.dependency_by_id(dep_id);
        let module_dependency = dependency.as_module_dependency();
        if (module_dependency.is_none() && dependency.as_context_dependency().is_none())
          || module_dependency.is_some_and(|module_dep| module_dep.weak())
          || module_graph.connection_by_dependency_id(dep_id).is_none()
        {
          continue;
        }

        if let Some(source_order) = dependency.source_order() {
          ordered_dependencies.push((source_order, *dep_id));
        } else {
          unordered_dependencies.push(*dep_id);
        }
      }
      ordered_dependencies.sort_by_key(|(source_order, _)| *source_order);

      let mut active_modules_by_block =
        HashMap::<DependenciesBlockIdentifier, IdentifierIndexMap<Vec<_>>>::default();
      for dep_id in ordered_dependencies
        .into_iter()
        .map(|(_, dep_id)| dep_id)
        .chain(unordered_dependencies)
      {
        let block = module_graph
          .get_parent_block(&dep_id)
          .map_or(DependenciesBlockIdentifier::Module(module), |block| {
            DependenciesBlockIdentifier::AsyncDependenciesBlock(*block)
          });
        let connection = module_graph
          .connection_by_dependency_id(&dep_id)
          .expect("should have connection");
        active_modules_by_block
          .entry(block)
          .or_default()
          .entry(*connection.module_identifier())
          .or_default()
          .push(connection);
      }

      for block in std::iter::once(DependenciesBlockIdentifier::Module(module)).chain(
        current_blocks
          .iter()
          .copied()
          .map(DependenciesBlockIdentifier::AsyncDependenciesBlock),
      ) {
        let mut outgoings = vec![];
        let active_modules = active_modules_by_block.remove(&block).unwrap_or_default();

        'outer: for (m, connections) in active_modules {
          let side_effects_state_artifact = &this_compilation
            .build_module_graph_artifact
            .side_effects_state_artifact;
          for conn in connections {
            if conn
              .active_state(
                module_graph,
                None,
                module_graph_cache,
                side_effects_state_artifact,
                &this_compilation.exports_info_artifact,
              )
              .is_not_false()
            {
              outgoings.push(m);
              continue 'outer;
            }
          }
        }

        let mut previous_modules = IdentifierIndexMap::default();
        let mut miss_in_previous = true;
        for modules in previous_modules_map.values() {
          let Some(outgoings) = modules.get(&block) else {
            continue;
          };
          miss_in_previous = false;

          for (outgoing, state, _) in outgoings.iter() {
            // Keep false connections to preserve source order.
            previous_modules
              .entry(*outgoing)
              .and_modify(|v| {
                if state.is_not_false() {
                  *v = *state;
                }
              })
              .or_insert(*state);
          }
        }

        if miss_in_previous {
          logger.log("new module detected, rebuilding chunk graph");
          return false;
        }

        if previous_modules
          .iter()
          .filter(|(_, conn_state)| conn_state.is_not_false())
          .map(|(m, _)| *m)
          .collect::<Vec<_>>()
          != outgoings
        {
          logger.log(format!("module outgoings change detected: {module}"));
          return false;
        }
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

  // Cache is not used, clear recovered artifact to avoid stale chunk graph data.
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
  Ok(())
}

impl ArtifactExt for BuildChunkGraphArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::BUILD_CHUNK_GRAPH;
  fn should_recover(incremental: &crate::incremental::Incremental) -> bool {
    incremental.passes_enabled(IncrementalPasses::BUILD_CHUNK_GRAPH)
  }
  fn recover(_incremental: &crate::incremental::Incremental, new: &mut Self, old: &mut Self) {
    new.code_splitter = mem::take(&mut old.code_splitter);
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
