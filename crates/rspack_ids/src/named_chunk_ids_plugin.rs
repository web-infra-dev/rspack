use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use rspack_collections::{DatabaseItem, UkeyIndexSet, UkeySet};
use rspack_core::{
  ChunkByUkey, ChunkGraph, ChunkGroupByUkey, ChunkNamedIdArtifact, ChunkUkey, CompilationChunkIds,
  Logger, ModuleGraph, ModuleGraphCacheArtifact, Plugin,
  chunk_graph_chunk::ChunkId,
  incremental::{self, IncrementalPasses, Mutation, Mutations},
};
use rspack_error::Diagnostic;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::itoa;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::id_helpers::{compare_chunks_natural, get_long_chunk_name, get_short_chunk_name};

#[tracing::instrument(skip_all)]
#[allow(clippy::too_many_arguments)]
fn assign_named_chunk_ids(
  chunks: UkeySet<ChunkUkey>,
  chunk_by_ukey: &mut ChunkByUkey,
  chunk_graph: &ChunkGraph,
  chunk_group_by_ukey: &ChunkGroupByUkey,
  module_ids_artifact: &rspack_core::ModuleIdsArtifact,
  context: &str,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  delimiter: &str,
  used_ids: &mut FxHashMap<ChunkId, ChunkUkey>,
  named_chunk_ids_artifact: &mut ChunkNamedIdArtifact,
  mutations: &mut Option<Mutations>,
) -> Vec<ChunkUkey> {
  let item_name_pair: Vec<_> = chunks
    .into_par_iter()
    .map(|item| {
      let chunk = chunk_by_ukey.expect_get(&item);
      let name = get_short_chunk_name(
        chunk,
        chunk_graph,
        context,
        delimiter,
        module_graph,
        module_graph_cache,
        named_chunk_ids_artifact,
      );
      (item, name)
    })
    .collect();
  let mut name_to_items: FxHashMap<String, UkeyIndexSet<ChunkUkey>> = FxHashMap::default();
  let mut invalid_and_repeat_names: FxHashSet<String> = std::iter::once(String::new()).collect();
  for (item, name) in item_name_pair {
    named_chunk_ids_artifact
      .chunk_short_names
      .insert(item, name.clone());

    let items = name_to_items.entry(name.clone()).or_default();
    items.insert(item);
    // If the short chunk id is conflict, then we need to rename all the conflicting chunks to long module id
    if items.len() > 1 {
      invalid_and_repeat_names.insert(name);
    }
    // Also rename the conflicting chunks in used_ids
    else if let Some(item) = used_ids.get(name.as_str())
    // Unless the chunk is explicitly using chunk name as id
      && matches!(chunk_by_ukey.expect_get(item).name(), Some(chunk_name) if chunk_name != name)
    {
      items.insert(*item);
      invalid_and_repeat_names.insert(name);
    }
  }

  let item_name_pair: Vec<(ChunkUkey, String)> = invalid_and_repeat_names
    .iter()
    .flat_map(|name| {
      let mut res = vec![];
      for item in name_to_items.remove(name).unwrap_or_default() {
        res.push((name.clone(), item));
      }
      res
    })
    .par_bridge()
    .map(|(_, item)| {
      let chunk = chunk_by_ukey.expect_get(&item);
      let long_name = get_long_chunk_name(
        chunk,
        chunk_graph,
        context,
        delimiter,
        module_graph,
        module_graph_cache,
        named_chunk_ids_artifact,
      );
      (item, long_name)
    })
    .collect();

  for (item, name) in item_name_pair {
    named_chunk_ids_artifact
      .chunk_long_names
      .insert(item, name.clone());

    let items = name_to_items.entry(name.clone()).or_default();
    items.insert(item);
    // Also rename the conflicting chunks in used_ids
    if let Some(item) = used_ids.get(name.as_str())
    // Unless the chunk is explicitly using chunk name as id
      && matches!(chunk_by_ukey.expect_get(item).name(), Some(chunk_name) if chunk_name != name)
    {
      items.insert(*item);
    }
  }

  let name_to_items_keys = name_to_items.keys().cloned().collect::<FxHashSet<_>>();
  let mut unnamed_items = vec![];

  let mut ordered_chunk_modules_cache = Default::default();

  // Sort by name to ensure deterministic processing order
  let mut name_to_items_sorted: Vec<_> = name_to_items.into_iter().collect();
  name_to_items_sorted.sort_by(|a, b| a.0.cmp(&b.0));

  for (name, mut items) in name_to_items_sorted {
    if name.is_empty() {
      for item in items {
        unnamed_items.push(item)
      }
    } else if items.len() == 1 && !used_ids.contains_key(name.as_str()) {
      let item = items[0];
      let chunk = chunk_by_ukey.expect_get_mut(&item);
      let name: ChunkId = name.into();
      if chunk.set_id(name.clone())
        && let Some(mutations) = mutations
      {
        mutations.add(Mutation::ChunkSetId { chunk: item });
      }
      used_ids.insert(name, item);
    } else {
      items.sort_unstable_by(|a, b| {
        let a = chunk_by_ukey.expect_get(a);
        let b = chunk_by_ukey.expect_get(b);
        compare_chunks_natural(
          chunk_graph,
          chunk_group_by_ukey,
          module_ids_artifact,
          a,
          b,
          &mut ordered_chunk_modules_cache,
        )
      });
      let mut i = 0;
      for item in items {
        let mut i_buffer = itoa::Buffer::new();
        let mut formatted_name = format!("{name}{}", i_buffer.format(i));
        while name_to_items_keys.contains(&formatted_name)
          && used_ids.contains_key(formatted_name.as_str())
        {
          i += 1;
          let mut i_buffer = itoa::Buffer::new();
          formatted_name = format!("{name}{}", i_buffer.format(i));
        }
        let chunk = chunk_by_ukey.expect_get_mut(&item);
        let name: ChunkId = formatted_name.into();
        if chunk.set_id(name.clone())
          && let Some(mutations) = mutations
        {
          mutations.add(Mutation::ChunkSetId { chunk: item });
        }
        used_ids.insert(name, item);
        i += 1;
      }
    }
  }
  unnamed_items.sort_unstable_by(|a, b| {
    let a = chunk_by_ukey.expect_get(a);
    let b = chunk_by_ukey.expect_get(b);
    compare_chunks_natural(
      chunk_graph,
      chunk_group_by_ukey,
      module_ids_artifact,
      a,
      b,
      &mut ordered_chunk_modules_cache,
    )
  });
  unnamed_items
}

#[plugin]
#[derive(Debug)]
pub struct NamedChunkIdsPlugin {
  pub delimiter: String,
  pub context: Option<String>,
}

impl NamedChunkIdsPlugin {
  pub fn new(delimiter: Option<String>, context: Option<String>) -> Self {
    Self::new_inner(delimiter.unwrap_or_else(|| "-".to_string()), context)
  }
}

#[plugin_hook(CompilationChunkIds for NamedChunkIdsPlugin)]
async fn chunk_ids(
  &self,
  compilation: &rspack_core::Compilation,
  chunk_by_ukey: &mut ChunkByUkey,
  named_chunk_ids_artifact: &mut ChunkNamedIdArtifact,
  _diagnostics: &mut Vec<Diagnostic>,
) -> rspack_error::Result<()> {
  // Clear artifact when incremental is disabled for CHUNK_IDS pass
  if !compilation
    .incremental
    .passes_enabled(IncrementalPasses::CHUNK_IDS)
  {
    named_chunk_ids_artifact.clear();
  }

  if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::CHUNK_IDS)
  {
    tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::CHUNK_IDS, %mutations);
    let mut affected_chunks: UkeySet<ChunkUkey> = UkeySet::default();
    for mutation in mutations.iter() {
      match mutation {
        Mutation::ChunkRemove { chunk } => {
          named_chunk_ids_artifact.remove(chunk);
        }
        Mutation::ModuleSetId { module } => {
          let chunks = compilation.chunk_graph.get_module_chunks(*module);
          affected_chunks.extend(chunks.iter().copied());
        }
        _ => {}
      }
    }

    named_chunk_ids_artifact
      .retain(|chunk| chunk_by_ukey.contains(chunk) && !affected_chunks.contains(chunk));
  }

  let mut chunks: UkeySet<ChunkUkey> = chunk_by_ukey
    .values_mut()
    .map(|chunk| {
      if let Some(id) = named_chunk_ids_artifact.chunk_ids.get(&chunk.ukey()) {
        chunk.set_id(id.clone());
      }
      chunk.ukey()
    })
    .collect();

  let chunks_len = chunks.len();

  let mut mutations = compilation
    .incremental
    .mutations_writeable()
    .then(Mutations::default);

  let mut used_ids: FxHashMap<ChunkId, ChunkUkey> = Default::default();

  // Use chunk name as default chunk id
  chunks.retain(|chunk_ukey| {
    let chunk = chunk_by_ukey.expect_get_mut(chunk_ukey);
    if let Some(chunk_name) = chunk.name() {
      let name = chunk_name.to_string();
      used_ids.insert(name.clone().into(), *chunk_ukey);
      if chunk.set_id(name)
        && let Some(mutations) = &mut mutations
      {
        mutations.add(Mutation::ChunkSetId { chunk: *chunk_ukey });
      }
      return false;
    }
    true
  });
  let named_chunks_len = chunks_len - chunks.len();

  let module_graph = compilation.get_module_graph();
  let context = compilation.options.context.as_str();
  let unnamed_chunks = assign_named_chunk_ids(
    chunks,
    chunk_by_ukey,
    &compilation.chunk_graph,
    &compilation.chunk_group_by_ukey,
    &compilation.module_ids_artifact,
    context,
    module_graph,
    &compilation.module_graph_cache_artifact,
    &self.delimiter,
    &mut used_ids,
    named_chunk_ids_artifact,
    &mut mutations,
  );

  if !unnamed_chunks.is_empty() {
    let mut next_id = 0;
    for chunk_ukey in &unnamed_chunks {
      let chunk = chunk_by_ukey.expect_get_mut(chunk_ukey);
      let mut id = next_id.to_string();
      while used_ids.contains_key(id.as_str()) {
        next_id += 1;
        id = next_id.to_string();
      }

      used_ids.insert(id.clone().into(), *chunk_ukey);
      if chunk.set_id(id)
        && let Some(mutations) = &mut mutations
      {
        mutations.add(Mutation::ChunkSetId { chunk: *chunk_ukey });
      }
      next_id += 1;
    }
  }

  if compilation
    .incremental
    .mutations_readable(IncrementalPasses::CHUNK_IDS)
    && let Some(mutations) = &mutations
  {
    let logger = compilation.get_logger("rspack.incremental.chunkIds");
    logger.log(format!(
      "{} chunks are updated by set_chunk_id, with {} chunks using name as id, and {} unnamed chunks",
      mutations.len(),
      named_chunks_len,
      unnamed_chunks.len(),
    ));
  }

  if let Some(mut compilation_mutations) = compilation.incremental.mutations_write()
    && let Some(mutations) = mutations
  {
    compilation_mutations.extend(mutations);
  }

  // store chunk id map to the artifact
  chunk_by_ukey.values().for_each(|chunk| {
    named_chunk_ids_artifact
      .chunk_ids
      .insert(chunk.ukey(), chunk.expect_id().clone());
  });

  Ok(())
}

impl Plugin for NamedChunkIdsPlugin {
  fn name(&self) -> &'static str {
    "rspack.NamedChunkIdsPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> rspack_error::Result<()> {
    ctx.compilation_hooks.chunk_ids.tap(chunk_ids::new(self));
    Ok(())
  }
}
