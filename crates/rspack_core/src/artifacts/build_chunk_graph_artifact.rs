use std::mem;

use futures::Future;
use itertools::Itertools;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_error::Result;
use rspack_util::{fx_hash::FxIndexMap, tracing_preset::TRACING_BENCH_TARGET};
use rustc_hash::FxHashMap as HashMap;
use tracing::instrument;

use crate::{
  ArtifactExt, ChunkByUkey, ChunkGraph, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, Compilation,
  Logger,
  build_chunk_graph::code_splitter::CodeSplitter,
  fast_set,
  incremental::{IncrementalPasses, Mutation},
};

// `sideEffects: true` and `usedExports` let an edit elsewhere turn on an import
// between two unchanged modules.
fn can_reuse_unreached_modules(compilation: &Compilation) -> bool {
  !compilation.options.optimization.side_effects.is_true()
    && !compilation.options.optimization.used_exports.is_enable()
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
  has_diagnostics: bool,
}

impl BuildChunkGraphArtifact {
  pub(crate) fn set_code_splitter(&mut self, code_splitter: CodeSplitter) {
    fast_set(&mut self.code_splitter, code_splitter);
  }

  // Module incoming edges are covered by the importing modules' outgoing changes.
  // Entry roots and options need a separate comparison.
  fn can_skip_rebuilding(&self, this_compilation: &Compilation) -> bool {
    self.can_skip_rebuilding_legacy(this_compilation)
  }

  fn can_skip_rebuilding_legacy(&self, this_compilation: &Compilation) -> bool {
    let logger = this_compilation.get_logger("rspack.Compilation.codeSplittingCache");

    if self.has_diagnostics {
      logger.log("previous code splitting emitted diagnostics, rebuilding chunk graph");
      return false;
    }

    if !this_compilation.entries.keys().eq(
      this_compilation
        .build_chunk_graph_artifact
        .entrypoints
        .keys(),
    ) {
      logger.log("entrypoints change detected, rebuilding chunk graph");
      return false;
    }

    let module_graph = this_compilation.get_module_graph();
    if !this_compilation
      .global_entry
      .dependencies
      .iter()
      .filter_map(|dependency| module_graph.module_identifier_by_dependency_id(dependency))
      .eq(self.code_splitter.global_entry_modules.iter())
    {
      logger.log("global entry roots change detected, rebuilding chunk graph");
      return false;
    }

    for (name, entry) in &this_compilation.entries {
      let entrypoint = self.chunk_group_by_ukey.expect_get(&self.entrypoints[name]);
      let modules = entry
        .dependencies
        .iter()
        .filter_map(|dependency| module_graph.module_identifier_by_dependency_id(dependency));
      let included_modules = this_compilation
        .global_entry
        .include_dependencies
        .iter()
        .filter_map(|dependency| module_graph.module_identifier_by_dependency_id(dependency))
        .sorted_unstable()
        .chain(
          entry
            .include_dependencies
            .iter()
            .filter_map(|dependency| module_graph.module_identifier_by_dependency_id(dependency))
            .sorted_unstable(),
        );
      if entrypoint.kind.get_entry_options() != Some(&entry.options)
        || !self
          .code_splitter
          .entry_modules
          .get(name)
          .is_some_and(|cached| modules.eq(cached.iter()))
        || !self
          .code_splitter
          .entry_include_modules
          .get(name)
          .is_some_and(|cached| included_modules.eq(cached.iter()))
      {
        logger.log(format!(
          "entry inputs change detected: {name}, rebuilding chunk graph"
        ));
        return false;
      }
    }

    if !self.async_entrypoints.is_empty()
      && !this_compilation
        .global_entry
        .include_dependencies
        .iter()
        .filter(|dependency| {
          !module_graph
            .dependency_by_id(dependency)
            .skip_async_entrypoints()
        })
        .filter_map(|dependency| module_graph.module_identifier_by_dependency_id(dependency))
        .eq(self.code_splitter.global_include_modules.iter())
    {
      logger.log("global includes change detected, rebuilding chunk graph");
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

    let affected_modules = mutations.get_affected_modules_with_module_graph(module_graph);
    let previous_modules_map = &this_compilation
      .build_chunk_graph_artifact
      .code_splitter
      .block_modules_runtime_map;

    if previous_modules_map.is_empty() {
      logger.log("no cache detected, rebuilding chunk graph");
      return false;
    }

    let reuse_unreached = can_reuse_unreached_modules(this_compilation);
    let rebuilt: IdentifierSet = if reuse_unreached {
      mutations
        .iter()
        .filter_map(|mutation| match mutation {
          Mutation::ModuleUpdate { module } | Mutation::ModuleAdd { module } => Some(*module),
          _ => None,
        })
        .collect()
    } else {
      IdentifierSet::default()
    };

    for module in affected_modules {
      if !self.code_splitter.can_reuse_affected_module(
        module,
        this_compilation,
        reuse_unreached && !rebuilt.contains(&module),
      ) {
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
    self.has_diagnostics = false;
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

  // The asset pass repeats this notice even when the chunk graph is reused.
  if compilation.entries.values().any(|entry| {
    entry
      .options
      .filename
      .as_ref()
      .is_some_and(|filename| filename.has_hash_placeholder())
  }) && let Some(Some(diagnostic)) = compilation.incremental.disable_passes(
    IncrementalPasses::CHUNK_ASSET,
    "Chunk filename that dependent on full hash",
    "chunk filename that dependent on full hash is not supported in incremental compilation",
  ) {
    compilation.push_diagnostic(diagnostic);
  }

  let diagnostics_len = compilation.diagnostics().len();
  let compilation = task(compilation).await?;
  compilation.build_chunk_graph_artifact.has_diagnostics =
    compilation.diagnostics().len() > diagnostics_len;
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
    new.has_diagnostics = old.has_diagnostics;
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
