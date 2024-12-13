use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use rspack_collections::{DatabaseItem, UkeyIndexSet, UkeyMap, UkeySet};
use rspack_core::{
  chunk_graph_chunk::ChunkId,
  incremental::{IncrementalPasses, Mutation, Mutations},
  ApplyContext, ChunkGraph, ChunkUkey, Compilation, CompilationChunkIds, CompilerOptions, Logger,
  Plugin, PluginContext,
};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::itoa;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::id_helpers::{compare_chunks_natural, get_long_chunk_name, get_short_chunk_name};

#[tracing::instrument(skip_all)]
fn assign_named_chunk_ids(
  chunks: UkeySet<ChunkUkey>,
  compilation: &Compilation,
  delimiter: &str,
  used_ids: &mut FxHashMap<ChunkId, ChunkUkey>,
  chunk_ids: &mut UkeyMap<ChunkUkey, ChunkId>,
  mutations: &mut Option<Mutations>,
) -> Vec<ChunkUkey> {
  let context: &str = compilation.options.context.as_ref();
  let module_graph = compilation.get_module_graph();
  let chunk_graph = &compilation.chunk_graph;

  let item_name_pair: Vec<_> = chunks
    .into_par_iter()
    .map(|item| {
      let chunk = compilation.chunk_by_ukey.expect_get(&item);
      let name = get_short_chunk_name(chunk, chunk_graph, context, delimiter, &module_graph);
      (item, name)
    })
    .collect();
  let mut name_to_items: FxHashMap<String, UkeyIndexSet<ChunkUkey>> = FxHashMap::default();
  let mut invalid_and_repeat_names: FxHashSet<String> = std::iter::once(String::new()).collect();
  for (item, name) in item_name_pair {
    let items = name_to_items.entry(name.clone()).or_default();
    items.insert(item);
    // If the short chunk id is conflict, then we need to rename all the conflicting chunks to long module id
    if items.len() > 1 {
      invalid_and_repeat_names.insert(name);
    }
    // Also rename the conflicting chunks in used_ids
    else if let Some(item) = used_ids.get(name.as_str())
    // Unless the chunk is explicitly using chunk name as id
      && matches!(compilation.chunk_by_ukey.expect_get(item).name(), Some(chunk_name) if chunk_name != name)
    {
      items.insert(*item);
      invalid_and_repeat_names.insert(name);
    }
  }

  let item_name_pair: Vec<_> = invalid_and_repeat_names
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
      let chunk = compilation.chunk_by_ukey.expect_get(&item);
      let long_name = get_long_chunk_name(chunk, chunk_graph, context, delimiter, &module_graph);
      (item, long_name)
    })
    .collect();
  for (item, name) in item_name_pair {
    let items = name_to_items.entry(name.clone()).or_default();
    items.insert(item);
    // Also rename the conflicting chunks in used_ids
    if let Some(item) = used_ids.get(name.as_str())
    // Unless the chunk is explicitly using chunk name as id
      && matches!(compilation.chunk_by_ukey.expect_get(item).name(), Some(chunk_name) if chunk_name != name)
    {
      items.insert(*item);
    }
  }

  let name_to_items_keys = name_to_items.keys().cloned().collect::<FxHashSet<_>>();
  let mut unnamed_items = vec![];

  for (name, mut items) in name_to_items {
    if name.is_empty() {
      for item in items {
        unnamed_items.push(item)
      }
    } else if items.len() == 1 && !used_ids.contains_key(name.as_str()) {
      let item = items[0];
      let name: ChunkId = name.into();
      if ChunkGraph::set_chunk_id(chunk_ids, item, name.clone())
        && let Some(mutations) = mutations
      {
        mutations.add(Mutation::ChunkSetId { chunk: item });
      }
      used_ids.insert(name, item);
    } else {
      items.sort_unstable_by(|a, b| {
        let a = compilation.chunk_by_ukey.expect_get(a);
        let b = compilation.chunk_by_ukey.expect_get(b);
        compare_chunks_natural(chunk_graph, &module_graph, &compilation.module_ids, a, b)
      });
      let mut i = 0;
      for item in items {
        let mut formatted_name = format!("{name}{}", itoa!(i));
        while name_to_items_keys.contains(&formatted_name)
          && used_ids.contains_key(formatted_name.as_str())
        {
          i += 1;
          formatted_name = format!("{name}{}", itoa!(i));
        }
        let name: ChunkId = formatted_name.into();
        if ChunkGraph::set_chunk_id(chunk_ids, item, name.clone())
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
    let a = compilation.chunk_by_ukey.expect_get(a);
    let b = compilation.chunk_by_ukey.expect_get(b);
    compare_chunks_natural(chunk_graph, &module_graph, &compilation.module_ids, a, b)
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
fn chunk_ids(&self, compilation: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
  let more_chunks = if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::CHUNK_IDS)
  {
    let mut affected_chunks: UkeySet<ChunkUkey> = UkeySet::default();
    for mutation in mutations.iter() {
      match mutation {
        Mutation::ChunkRemove { chunk } => {
          compilation.chunk_ids.remove(chunk);
        }
        Mutation::ModuleSetId { module } => {
          affected_chunks.extend(compilation.chunk_graph.get_module_chunks(*module));
        }
        _ => {}
      }
    }
    compilation
      .chunk_ids
      .retain(|chunk, _| compilation.chunk_by_ukey.contains(chunk));
    affected_chunks
  } else {
    UkeySet::default()
  };

  let mut chunks: UkeySet<ChunkUkey> = compilation
    .chunk_by_ukey
    .values()
    .filter(|chunk| chunk.id(&compilation.chunk_ids).is_none())
    .map(|chunk| chunk.ukey())
    .collect();
  chunks.extend(more_chunks);
  let chunks_len = chunks.len();

  let mut mutations = compilation
    .incremental
    .can_write_mutations()
    .then(Mutations::default);

  // Use chunk name as default chunk id
  chunks.retain(|chunk_ukey| {
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
    if let Some(chunk_name) = chunk.name() {
      if ChunkGraph::set_chunk_id(&mut compilation.chunk_ids, *chunk_ukey, chunk_name.into())
        && let Some(mutations) = &mut mutations
      {
        mutations.add(Mutation::ChunkSetId { chunk: *chunk_ukey });
      }
      return false;
    }
    true
  });
  let named_chunks_len = chunks_len - chunks.len();

  let mut chunk_ids = std::mem::take(&mut compilation.chunk_ids);
  let mut used_ids: FxHashMap<ChunkId, ChunkUkey> = chunk_ids
    .iter()
    .map(|(&chunk, id)| (id.clone(), chunk))
    .collect();

  let unnamed_chunks = assign_named_chunk_ids(
    chunks,
    compilation,
    &self.delimiter,
    &mut used_ids,
    &mut chunk_ids,
    &mut mutations,
  );

  if !unnamed_chunks.is_empty() {
    let mut next_id = 0;
    for chunk_ukey in &unnamed_chunks {
      let chunk = compilation.chunk_by_ukey.expect_get_mut(chunk_ukey);
      let mut id = next_id.to_string();
      while used_ids.contains_key(id.as_str()) {
        next_id += 1;
        id = next_id.to_string();
      }
      if chunk.set_id(&mut chunk_ids, id)
        && let Some(mutations) = &mut mutations
      {
        mutations.add(Mutation::ChunkSetId { chunk: *chunk_ukey });
      }
      next_id += 1;
    }
  }

  if compilation
    .incremental
    .can_read_mutations(IncrementalPasses::CHUNK_IDS)
    && let Some(mutations) = &mutations
  {
    let logger = compilation.get_logger("rspack.incremental.chunkIds");
    logger.log(format!(
      "{} chunks are affected, {} in total",
      chunks_len,
      compilation.chunk_by_ukey.len(),
    ));
    logger.log(format!(
      "{} chunks are updated by set_chunk_id, with {} chunks using name as id, and {} unnamed chunks",
      mutations.len(),
      named_chunks_len,
      unnamed_chunks.len(),
    ));
  }

  if let Some(compilation_mutations) = compilation.incremental.mutations_write()
    && let Some(mutations) = mutations
  {
    compilation_mutations.extend(mutations);
  }

  compilation.chunk_ids = chunk_ids;
  Ok(())
}

impl Plugin for NamedChunkIdsPlugin {
  fn name(&self) -> &'static str {
    "rspack.NamedChunkIdsPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &CompilerOptions,
  ) -> rspack_error::Result<()> {
    ctx
      .context
      .compilation_hooks
      .chunk_ids
      .tap(chunk_ids::new(self));
    Ok(())
  }
}
