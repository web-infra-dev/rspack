use rayon::prelude::*;
use rspack_core::{
  ChunkByUkey, ChunkNamedIdArtifact, Compilation, CompilationChunkIds, Plugin,
  incremental::IncrementalPasses,
};
use rspack_error::{Diagnostic, Result, error};
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};

use crate::{
  compat_hashed_id::{
    CompatHashedIdAssigner, FULL_LOWERCASE_ALPHANUMERIC_LENGTH, hash_lowercase_alphanumeric,
    normalize_min_length, validate_min_length,
  },
  id_helpers::{
    NaturalChunkCompareCache, compare_chunks_natural, get_full_chunk_name, get_used_chunk_ids,
  },
};

#[derive(Debug, Clone, Default)]
pub struct CompatHashedChunkIdsPluginOptions {
  pub min_length: Option<usize>,
}

#[plugin]
#[derive(Debug)]
pub struct CompatHashedChunkIdsPlugin {
  min_length: usize,
}

impl Default for CompatHashedChunkIdsPlugin {
  fn default() -> Self {
    Self::new(Default::default())
  }
}

impl CompatHashedChunkIdsPlugin {
  pub fn new(options: CompatHashedChunkIdsPluginOptions) -> Self {
    Self::new_inner(normalize_min_length(options.min_length))
  }
}

#[plugin_hook(CompilationChunkIds for CompatHashedChunkIdsPlugin)]
async fn chunk_ids(
  &self,
  compilation: &Compilation,
  chunk_by_ukey: &mut ChunkByUkey,
  _named_chunk_ids_artifact: &mut ChunkNamedIdArtifact,
  diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::CHUNK_IDS | IncrementalPasses::MODULES_HASHES,
    "CompatHashedChunkIdsPlugin (optimization.chunkIds = \"compat-hashed\")",
    "it requires calculating the id of all the chunks, which is a global effect",
  ) && let Some(diagnostic) = diagnostic
  {
    diagnostics.push(diagnostic);
  }

  validate_min_length(
    self.min_length,
    FULL_LOWERCASE_ALPHANUMERIC_LENGTH,
    "CompatHashedChunkIdsPlugin",
  )?;

  // Prevent generated ids from aliasing preassigned ids on case-insensitive file systems.
  let used_ids = get_used_chunk_ids(chunk_by_ukey)
    .into_iter()
    .map(|mut id| {
      id.make_ascii_lowercase();
      id
    })
    .collect::<FxHashSet<_>>();
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  let module_graph = compilation.get_module_graph();
  let module_graph_cache = &compilation.module_graph_cache_artifact;
  let context = compilation.options.context.as_ref();
  let chunks = chunk_by_ukey
    .values()
    .filter(|chunk| chunk.id().is_none())
    .collect::<Vec<_>>();

  let mut chunks_with_hashes = chunks
    .into_par_iter()
    .map(|chunk| {
      let name = get_full_chunk_name(
        chunk,
        chunk_graph,
        module_graph,
        module_graph_cache,
        &compilation
          .build_module_graph_artifact
          .side_effects_state_artifact,
        context,
        &compilation.exports_info_artifact,
      );
      (chunk, hash_lowercase_alphanumeric(&name))
    })
    .collect::<Vec<_>>();

  let mut chunk_compare_cache = NaturalChunkCompareCache::default();
  chunks_with_hashes.sort_unstable_by(|(a, _), (b, _)| {
    compare_chunks_natural(
      chunk_graph,
      &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
      &compilation.module_ids_artifact,
      a,
      b,
      &mut chunk_compare_cache,
    )
  });

  let mut chunk_key_to_id =
    FxHashMap::with_capacity_and_hasher(chunks_with_hashes.len(), FxBuildHasher::default());
  let mut id_assigner = CompatHashedIdAssigner::new(self.min_length, used_ids);
  for (chunk, hash) in chunks_with_hashes {
    let Some(chunk_id) = id_assigner.assign(&hash) else {
      return Err(error!(
        "Unable to assign a unique compat-hashed id to chunk '{:?}' after using all {FULL_LOWERCASE_ALPHANUMERIC_LENGTH} hash characters",
        chunk.ukey()
      ));
    };

    chunk_key_to_id.insert(chunk.ukey(), chunk_id);
  }

  for (chunk_ukey, id) in chunk_key_to_id {
    chunk_by_ukey.expect_get_mut(&chunk_ukey).set_id(id);
  }

  Ok(())
}

impl Plugin for CompatHashedChunkIdsPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compilation_hooks.chunk_ids.tap(chunk_ids::new(self));
    Ok(())
  }
}
