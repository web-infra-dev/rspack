use rspack_core::{
  ChunkByUkey, ChunkNamedIdArtifact, CompilationChunkIds, Plugin, incremental::IncrementalPasses,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxBuildHasher, FxHashMap};

use crate::id_helpers::{
  assign_deterministic_ids_from_precomputed_candidates, compare_chunks_natural,
  get_deterministic_id_range, get_full_chunk_name, get_used_chunk_ids,
  precompute_deterministic_id_candidates,
};

#[plugin]
#[derive(Debug, Default)]
pub struct DeterministicChunkIdsPlugin {
  pub delimiter: String,
  pub context: Option<String>,
}

impl DeterministicChunkIdsPlugin {
  pub fn new(delimiter: Option<String>, context: Option<String>) -> Self {
    Self::new_inner(delimiter.unwrap_or_else(|| "~".to_string()), context)
  }
}

#[plugin_hook(CompilationChunkIds for DeterministicChunkIdsPlugin)]
async fn chunk_ids(
  &self,
  compilation: &rspack_core::Compilation,
  chunk_by_ukey: &mut ChunkByUkey,
  _named_chunk_ids_artifact: &mut ChunkNamedIdArtifact,
  diagnostics: &mut Vec<Diagnostic>,
) -> rspack_error::Result<()> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::CHUNK_IDS,
    "DeterministicChunkIdsPlugin (optimization.chunkIds = \"deterministic\")",
    "it requires calculating the id of all the chunks, which is a global effect",
  ) && let Some(diagnostic) = diagnostic
  {
    diagnostics.push(diagnostic);
  }

  let mut used_ids = get_used_chunk_ids(chunk_by_ukey);
  let used_ids_len = used_ids.len();

  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  let module_graph = compilation.get_module_graph();
  let module_graph_cache = &compilation.module_graph_cache_artifact;
  let context = self
    .context
    .clone()
    .unwrap_or_else(|| compilation.options.context.as_str().to_string());

  let max_length = 3;
  let expand_factor = 10;
  let salt = 10;

  let chunks = chunk_by_ukey
    .values()
    .filter(|chunk| chunk.id().is_none())
    .collect::<Vec<_>>();
  let mut chunk_key_to_id =
    FxHashMap::with_capacity_and_hasher(chunks.len(), FxBuildHasher::default());

  let mut ordered_chunk_modules_cache = Default::default();
  let range = get_deterministic_id_range(
    chunks.len(),
    &[usize::pow(10, max_length)],
    expand_factor,
    used_ids_len,
  );
  let prepared_chunks = precompute_deterministic_id_candidates(
    chunks,
    |chunk| {
      get_full_chunk_name(
        chunk,
        chunk_graph,
        module_graph,
        module_graph_cache,
        &compilation
          .build_module_graph_artifact
          .side_effects_state_artifact,
        &context,
        &compilation.exports_info_artifact,
      )
    },
    |a, b| {
      compare_chunks_natural(
        chunk_graph,
        &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
        &compilation.module_ids_artifact,
        a,
        b,
        &mut ordered_chunk_modules_cache,
      )
    },
    range,
    salt,
  );

  assign_deterministic_ids_from_precomputed_candidates(
    prepared_chunks,
    &mut used_ids,
    range,
    salt,
    |chunk, id| {
      chunk_key_to_id.insert(chunk.ukey(), id);
    },
  );

  for (chunk_ukey, id) in chunk_key_to_id {
    let chunk = chunk_by_ukey.expect_get_mut(&chunk_ukey);
    chunk.set_id(id.to_string());
  }

  Ok(())
}

impl Plugin for DeterministicChunkIdsPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compilation_hooks.chunk_ids.tap(chunk_ids::new(self));
    Ok(())
  }
}
