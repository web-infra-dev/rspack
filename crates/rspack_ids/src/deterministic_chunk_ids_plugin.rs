use std::sync::{Arc, atomic::Ordering::Relaxed};

use rayon::prelude::*;
use rspack_core::{
  ChunkByUkey, ChunkIdIdentity, ChunkNamedIdArtifact, CompilationChunkIds,
  DeterministicChunkIdsInputs, Plugin, incremental::IncrementalPasses,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxBuildHasher, FxHashMap};

use crate::id_helpers::{
  NaturalChunkCompareCache, assign_deterministic_ids_sorted, compare_chunks_natural,
  get_deterministic_id_range, get_full_chunk_name, get_used_chunk_ids,
};

#[plugin]
#[derive(Debug, Default)]
pub struct DeterministicChunkIdsPlugin {
  pub delimiter: String,
  pub context: Option<String>,
}

const INITIAL_ID_RANGES: [usize; 1] = [1000];

#[inline(never)]
fn compare_reserved_ids(a: &rspack_core::ChunkId, b: &rspack_core::ChunkId) -> std::cmp::Ordering {
  a.cmp(b)
}

impl DeterministicChunkIdsPlugin {}

#[plugin_hook(CompilationChunkIds for DeterministicChunkIdsPlugin)]
async fn chunk_ids(
  &self,
  compilation: &rspack_core::Compilation,
  chunk_by_ukey: &mut ChunkByUkey,
  _named_chunk_ids_artifact: &mut ChunkNamedIdArtifact,
  diagnostics: &mut Vec<Diagnostic>,
) -> rspack_error::Result<()> {
  let track_ids = compilation
    .incremental
    .passes_enabled(IncrementalPasses::MODULES_HASHES);
  compilation
    .chunk_ids_diff_artifact
    .track_effective_ids
    .store(track_ids, Relaxed);
  let hooks = &compilation.plugin_driver.compilation_hooks;
  let can_reuse = hooks.chunk_ids.tap_stages().len() == 1
    && !hooks.chunk_ids.has_interceptors()
    && compilation
      .incremental
      .passes_enabled(IncrementalPasses::CHUNK_IDS);
  // Other taps may alter reservations or allocation order. Keep their replay,
  // but final ID differences still drive module hashing in ChunkIdsPass.
  if !can_reuse
    && let Some(Some(diagnostic)) = compilation.incremental.disable_passes(
      IncrementalPasses::CHUNK_IDS,
      "DeterministicChunkIdsPlugin (optimization.chunkIds = \"deterministic\")",
      "its chunk ID hook chain has additional taps or interceptors",
    )
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

  let expand_factor = 10;
  let salt = 10;

  // Only a single known tap may share pre-hook identities with the final diff.
  // Opaque chains build them after all taps, conservatively covering mutations.
  let identities = if track_ids && can_reuse {
    chunk_by_ukey
      .values()
      .map(|chunk| {
        (
          chunk.ukey(),
          Arc::new(ChunkIdIdentity::new(chunk, chunk_graph)),
        )
      })
      .collect::<FxHashMap<_, _>>()
  } else {
    FxHashMap::default()
  };
  let mut chunks = chunk_by_ukey
    .values()
    .filter(|chunk| chunk.id().is_none())
    .collect::<Vec<_>>();
  let mut chunk_key_to_id =
    FxHashMap::with_capacity_and_hasher(chunks.len(), FxBuildHasher::default());

  let chunk_names = chunks
    .par_iter()
    .map(|chunk| -> Result<_> {
      Ok((
        chunk.ukey(),
        get_full_chunk_name(
          chunk,
          chunk_graph,
          &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
          module_graph,
          module_graph_cache,
          &compilation
            .build_module_graph_artifact
            .side_effects_state_artifact,
          &context,
          &compilation.exports_info_artifact,
        )?,
      ))
    })
    .collect::<Result<FxHashMap<_, _>>>()?;

  let mut chunk_compare_cache = NaturalChunkCompareCache::default();
  chunks.sort_unstable_by(|a, b| {
    compare_chunks_natural(
      chunk_graph,
      &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
      &compilation.module_ids_artifact,
      a,
      b,
      &mut chunk_compare_cache,
    )
  });
  let inputs = (track_ids && can_reuse).then(|| {
    let mut reserved_ids = used_ids
      .iter()
      .map(|id| id.as_str().into())
      .collect::<Vec<_>>();
    reserved_ids.sort_unstable_by(compare_reserved_ids);
    DeterministicChunkIdsInputs {
      candidates: chunks
        .iter()
        .map(|chunk| {
          (
            Arc::clone(
              identities
                .get(&chunk.ukey())
                .expect("should have chunk identity"),
            ),
            chunk_names
              .get(&chunk.ukey())
              .expect("should have full chunk name")
              .clone(),
          )
        })
        .collect(),
      reserved_ids,
      context: context.clone(),
      delimiter: self.delimiter.clone(),
      range: get_deterministic_id_range(
        chunks.len(),
        &INITIAL_ID_RANGES,
        expand_factor,
        used_ids_len,
      ),
    }
  });
  let reused_ids = if let Some(inputs) = &inputs
    && compilation
      .incremental
      .mutations_readable(IncrementalPasses::CHUNK_IDS)
    && compilation
      .chunk_ids_diff_artifact
      .deterministic_inputs
      .as_ref()
      == Some(inputs)
  {
    let mut ids = FxHashMap::default();
    for (identity, id) in &compilation.chunk_ids_diff_artifact.previous_ids {
      ids
        .entry(identity)
        .and_modify(|id| *id = None)
        .or_insert(Some(id));
    }
    inputs
      .candidates
      .iter()
      .zip(&chunks)
      .map(|((identity, _), chunk)| {
        ids
          .get(identity)
          .copied()
          .flatten()
          .map(|id| (chunk.ukey(), id.clone()))
      })
      .collect::<Option<Vec<_>>>()
  } else {
    None
  };
  if track_ids {
    *compilation
      .chunk_ids_diff_artifact
      .pending_identities
      .lock()
      .expect("Mutex poisoned: chunk ID identities") = identities;
  }
  if let Some(reused_ids) = reused_ids {
    for (ukey, id) in reused_ids {
      chunk_by_ukey.expect_get_mut(&ukey).set_id(id);
    }
    *compilation
      .chunk_ids_diff_artifact
      .pending_inputs
      .lock()
      .expect("Mutex poisoned: deterministic chunk ID inputs") = inputs;
    return Ok(());
  }

  assign_deterministic_ids_sorted(
    chunks,
    |chunk| {
      chunk_names
        .get(&chunk.ukey())
        .expect("should have generated full chunk name")
        .as_str()
    },
    |chunk, id| {
      let size = used_ids.len();
      used_ids.insert(id.to_string());
      if used_ids.len() == size {
        return false;
      }

      chunk_key_to_id.insert(chunk.ukey(), id);
      true
    },
    &INITIAL_ID_RANGES,
    expand_factor,
    used_ids_len,
    salt,
  );

  for (chunk_ukey, id) in chunk_key_to_id {
    let chunk = chunk_by_ukey.expect_get_mut(&chunk_ukey);
    chunk.set_id(id.to_string());
  }

  if track_ids {
    *compilation
      .chunk_ids_diff_artifact
      .pending_inputs
      .lock()
      .expect("Mutex poisoned: deterministic chunk ID inputs") = inputs;
  }
  Ok(())
}

impl Plugin for DeterministicChunkIdsPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compilation_hooks.chunk_ids.tap(chunk_ids::new(self));
    Ok(())
  }
}
