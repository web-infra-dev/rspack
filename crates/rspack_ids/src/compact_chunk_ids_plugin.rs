use rayon::prelude::*;
use rspack_core::{
  ChunkByUkey, ChunkNamedIdArtifact, Compilation, CompilationChunkIds, Plugin,
  incremental::IncrementalPasses,
};
use rspack_error::{Diagnostic, Result, error};
use rspack_hash::{HashFunction, RspackHasher};
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxBuildHasher, FxHashMap};

use crate::{
  compact_id::{FULL_IDENTIFIER_LENGTH, encode_identifier_hash},
  id_helpers::{
    NaturalChunkCompareCache, compare_chunks_natural, get_full_chunk_name, get_used_chunk_ids,
  },
};

#[derive(Debug, Clone, Default)]
pub struct CompactChunkIdsPluginOptions {
  pub min_length: Option<usize>,
}

#[plugin]
#[derive(Debug)]
pub struct CompactChunkIdsPlugin {
  min_length: usize,
}

impl Default for CompactChunkIdsPlugin {
  fn default() -> Self {
    Self::new(Default::default())
  }
}

impl CompactChunkIdsPlugin {
  pub fn new(options: CompactChunkIdsPluginOptions) -> Self {
    Self::new_inner(
      options
        .min_length
        .filter(|min_length| *min_length != 0)
        .unwrap_or(1),
    )
  }
}

#[plugin_hook(CompilationChunkIds for CompactChunkIdsPlugin)]
async fn chunk_ids(
  &self,
  compilation: &Compilation,
  chunk_by_ukey: &mut ChunkByUkey,
  _named_chunk_ids_artifact: &mut ChunkNamedIdArtifact,
  diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::CHUNK_IDS | IncrementalPasses::MODULES_HASHES,
    "CompactChunkIdsPlugin (optimization.chunkIds = \"compact\")",
    "it requires calculating the id of all the chunks, which is a global effect",
  ) && let Some(diagnostic) = diagnostic
  {
    diagnostics.push(diagnostic);
  }

  if self.min_length > FULL_IDENTIFIER_LENGTH {
    return Err(error!(
      "'minLength' must not exceed {FULL_IDENTIFIER_LENGTH} for CompactChunkIdsPlugin"
    ));
  }

  let mut used_ids = get_used_chunk_ids(chunk_by_ukey);
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
      let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
      hasher.write(name.as_bytes());
      (chunk, encode_identifier_hash(hasher.finish()))
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
  for (chunk, hash) in chunks_with_hashes {
    // SAFETY: `encode_identifier_hash` only emits ASCII characters.
    let hash = unsafe { std::str::from_utf8_unchecked(&hash) };
    let Some(chunk_id) = (self.min_length..=hash.len()).find_map(|length| {
      let candidate = &hash[..length];
      if used_ids.contains(candidate) {
        None
      } else {
        Some(candidate.to_string())
      }
    }) else {
      return Err(error!(
        "Unable to assign a unique compact id to chunk '{:?}' after using all {FULL_IDENTIFIER_LENGTH} hash characters",
        chunk.ukey()
      ));
    };

    used_ids.insert(chunk_id.clone());
    chunk_key_to_id.insert(chunk.ukey(), chunk_id);
  }

  for (chunk_ukey, id) in chunk_key_to_id {
    chunk_by_ukey.expect_get_mut(&chunk_ukey).set_id(id);
  }

  Ok(())
}

impl Plugin for CompactChunkIdsPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compilation_hooks.chunk_ids.tap(chunk_ids::new(self));
    Ok(())
  }
}
