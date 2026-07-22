use std::mem;

use futures::Future;
use rspack_collections::IdentifierMap;
use rspack_error::Result;
use rspack_util::{fx_hash::FxIndexMap, tracing_preset::TRACING_BENCH_TARGET};
use rustc_hash::FxHashMap as HashMap;
use tracing::instrument;

use crate::{
  ArtifactExt, AsyncDependenciesBlockIdentifier, ChunkByUkey, ChunkGraph, ChunkGroupByUkey,
  ChunkGroupUkey, ChunkUkey, Compilation, DependencyLocation, Logger, ModuleIdentifier,
  build_chunk_graph::code_splitter::CodeSplitter,
  fast_set,
  incremental::{IncrementalPasses, Mutation},
};

struct ChunkGroupOriginUpdate {
  block: AsyncDependenciesBlockIdentifier,
  module: ModuleIdentifier,
  loc: Option<DependencyLocation>,
  request: Option<String>,
}

#[derive(Default)]
struct CodeSplittingReusePlan {
  origin_updates: Vec<ChunkGroupOriginUpdate>,
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
}

impl BuildChunkGraphArtifact {
  pub(crate) fn set_code_splitter(&mut self, code_splitter: CodeSplitter) {
    fast_set(&mut self.code_splitter, code_splitter);
  }

  fn plan_code_splitting_reuse(
    &self,
    this_compilation: &Compilation,
  ) -> Option<CodeSplittingReusePlan> {
    let logger = this_compilation.get_logger("rspack.Compilation.codeSplittingCache");

    if !this_compilation.entries.keys().eq(
      this_compilation
        .build_chunk_graph_artifact
        .entrypoints
        .keys(),
    ) {
      logger.log("entrypoints change detected, rebuilding chunk graph");
      return None;
    }

    let Some(mutations) = this_compilation
      .incremental
      .mutations_read(IncrementalPasses::BUILD_MODULE_GRAPH)
    else {
      logger.log("incremental for build module graph disabled, rebuilding chunk graph");
      // if disable incremental for build module graph phase, we can't skip rebuilding
      return None;
    };

    if mutations
      .iter()
      .any(|mutation| matches!(mutation, Mutation::ModuleRemove { .. }))
    {
      logger.log("module removal detected, rebuilding chunk graph");
      return None;
    }

    let module_graph = this_compilation.get_module_graph();
    let affected_modules = mutations
      .get_affected_modules_with_module_graph(module_graph)
      .into_iter()
      .collect::<Vec<_>>();
    let previous_splitter = &self.code_splitter;

    if previous_splitter.ordinal_by_module.is_empty() {
      logger.log("no cache detected, rebuilding chunk graph");
      return None;
    }

    if affected_modules
      .iter()
      .any(|module| !previous_splitter.ordinal_by_module.contains_key(module))
    {
      logger.log("new module detected, rebuilding chunk graph");
      return None;
    }

    let mut current_splitter = CodeSplitter::default();
    if current_splitter
      .prepare(&affected_modules, this_compilation)
      .is_err()
    {
      logger.log("failed to prepare current block values, rebuilding chunk graph");
      return None;
    }

    let mut plan = CodeSplittingReusePlan::default();
    for module in affected_modules {
      if !previous_splitter.module_code_splitting_value_equal(
        &mut current_splitter,
        module,
        this_compilation,
      ) {
        logger.log(format!(
          "module code splitting value changed: {module}, rebuilding chunk graph"
        ));
        return None;
      }

      let module = module_graph
        .module_by_identifier(&module)
        .expect("affected module should exist");
      for block_id in module.get_blocks() {
        let Some(origin_index) = previous_splitter.block_origin_indices.get(block_id) else {
          continue;
        };
        let Some(chunk_group_ukey) = self
          .chunk_graph
          .block_to_chunk_group_ukey
          .get(block_id)
          .copied()
        else {
          logger.log("cached block chunk group is missing, rebuilding chunk graph");
          return None;
        };
        let chunk_group = self.chunk_group_by_ukey.expect_get(&chunk_group_ukey);
        if chunk_group.origins().get(*origin_index).is_none() {
          logger.log("cached block origin is missing, rebuilding chunk graph");
          return None;
        }

        let block = module_graph.block_by_id_expect(block_id);
        plan.origin_updates.push(ChunkGroupOriginUpdate {
          block: *block_id,
          module: *block.parent(),
          loc: block.loc(),
          request: block.request().clone(),
        });
      }
    }

    Some(plan)
  }

  fn apply_code_splitting_reuse(&mut self, plan: CodeSplittingReusePlan) {
    for update in plan.origin_updates {
      let chunk_group_ukey = *self
        .chunk_graph
        .block_to_chunk_group_ukey
        .get(&update.block)
        .expect("validated block chunk group should exist");
      let origin_index = *self
        .code_splitter
        .block_origin_indices
        .get(&update.block)
        .expect("validated block origin should exist");
      self
        .chunk_group_by_ukey
        .expect_get_mut(&chunk_group_ukey)
        .update_origin(
          origin_index,
          Some(update.module),
          update.loc,
          update.request,
        );
    }
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
  let reuse_plan = incremental_code_splitting
    .then(|| {
      compilation
        .build_chunk_graph_artifact
        .plan_code_splitting_reuse(compilation)
    })
    .flatten();

  if let Some(reuse_plan) = reuse_plan {
    compilation
      .build_chunk_graph_artifact
      .apply_code_splitting_reuse(reuse_plan);

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
