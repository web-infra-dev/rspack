mod chunk_combination;

use std::collections::{HashMap, HashSet};

use chunk_combination::{ChunkCombination, ChunkCombinationBucket, ChunkCombinationUkey};
use rspack_core::{
  compare_chunks_with_graph, get_chunk_from_ukey, get_chunk_group_from_ukey, ChunkSizeOptions,
  ChunkUkey, Compilation, CompilationOptimizeChunks, Plugin,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

fn add_to_set_map(
  map: &mut HashMap<ChunkUkey, HashSet<ChunkCombinationUkey>>,
  key: &ChunkUkey,
  value: ChunkCombinationUkey,
) {
  if map.get(key).is_none() {
    let mut set = HashSet::new();
    set.insert(value);
    map.insert(*key, set);
  } else {
    let set = map.get_mut(key);
    if let Some(set) = set {
      set.insert(value);
    } else {
      let mut set = HashSet::new();
      set.insert(value);
      map.insert(*key, set);
    }
  }
}

#[derive(Debug, Clone, Default)]
pub struct LimitChunkCountPluginOptions {
  // Constant overhead for a chunk.
  pub chunk_overhead: Option<f64>,
  //  Multiplicator for initial chunks.
  pub entry_chunk_multiplicator: Option<f64>,
  // Limit the maximum number of chunks using a value greater greater than or equal to 1.
  pub max_chunks: usize,
}

#[plugin]
#[derive(Debug)]
pub struct LimitChunkCountPlugin {
  options: LimitChunkCountPluginOptions,
}

impl LimitChunkCountPlugin {
  pub fn new(options: LimitChunkCountPluginOptions) -> Self {
    Self::new_inner(options)
  }
}

#[plugin_hook(CompilationOptimizeChunks for LimitChunkCountPlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_ADVANCED)]
fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let max_chunks = self.options.max_chunks;
  if max_chunks < 1 {
    return Ok(None);
  }

  let mut chunks_ukeys = compilation
    .chunk_by_ukey
    .keys()
    .copied()
    .collect::<Vec<_>>();
  if chunks_ukeys.len() <= max_chunks {
    return Ok(None);
  }

  let chunk_by_ukey = &compilation.chunk_by_ukey.clone();
  let chunk_group_by_ukey = &compilation.chunk_group_by_ukey.clone();
  let chunk_graph = &compilation.chunk_graph.clone();
  let mut new_chunk_by_ukey = std::mem::take(&mut compilation.chunk_by_ukey);
  let mut new_chunk_group_by_ukey = std::mem::take(&mut compilation.chunk_group_by_ukey);
  let mut new_chunk_graph = std::mem::take(&mut compilation.chunk_graph);

  //    let chunk_graph = &compilation.chunk_graph.clone();
  let module_graph = compilation.get_module_graph();
  let mut remaining_chunks_to_merge = (chunks_ukeys.len() - max_chunks) as i64;

  // order chunks in a deterministic way
  chunks_ukeys.sort_by(|a, b| compare_chunks_with_graph(chunk_graph, &module_graph, a, b));

  // create a lazy sorted data structure to keep all combinations
  // this is large. Size = chunks * (chunks - 1) / 2
  // It uses a multi layer bucket sort plus normal sort in the last layer
  // It's also lazy so only accessed buckets are sorted
  let mut combinations = ChunkCombinationBucket::new();

  // we keep a mapping from chunk to all combinations
  // but this mapping is not kept up-to-date with deletions
  // so `deleted` flag need to be considered when iterating this
  let mut combinations_by_chunk: HashMap<ChunkUkey, HashSet<ChunkCombinationUkey>> = HashMap::new();

  let chunk_size_option = ChunkSizeOptions {
    chunk_overhead: self.options.chunk_overhead,
    entry_chunk_multiplicator: self.options.entry_chunk_multiplicator,
  };

  for (b_idx, b) in chunks_ukeys.iter().enumerate() {
    for (a_idx, a) in chunks_ukeys.iter().enumerate().take(b_idx) {
      if !chunk_graph.can_chunks_be_integrated(a, b, chunk_by_ukey, chunk_group_by_ukey) {
        continue;
      }

      let integrated_size = chunk_graph.get_integrated_chunks_size(
        a,
        b,
        &chunk_size_option,
        chunk_by_ukey,
        chunk_group_by_ukey,
        &module_graph,
        compilation,
      );
      let a_size = chunk_graph.get_chunk_size(
        a,
        &chunk_size_option,
        chunk_by_ukey,
        chunk_group_by_ukey,
        &module_graph,
        compilation,
      );
      let b_size = chunk_graph.get_chunk_size(
        b,
        &chunk_size_option,
        chunk_by_ukey,
        chunk_group_by_ukey,
        &module_graph,
        compilation,
      );

      let c = ChunkCombination {
        ukey: ChunkCombinationUkey::new(),
        deleted: false,
        size_diff: a_size + b_size - integrated_size,
        integrated_size,
        a: *a,
        b: *b,
        a_idx,
        b_idx,
        a_size,
        b_size,
      };

      add_to_set_map(&mut combinations_by_chunk, a, c.ukey);
      add_to_set_map(&mut combinations_by_chunk, b, c.ukey);
      combinations.add(c);
    }
  }

  // list of modified chunks during this run
  // combinations affected by this change are skipped to allow
  // further optimizations
  let mut modified_chunks: HashSet<ChunkUkey> = HashSet::new();

  while let Some(combination_ukey) = combinations.pop_first() {
    let combination = combinations.get_mut(&combination_ukey);
    combination.deleted = true;
    let a = combination.a;
    let b = combination.b;
    let integrated_size = combination.integrated_size;

    // skip over pair when
    // one of the already merged chunks is a parent of one of the chunks
    if !modified_chunks.is_empty() {
      let a_chunk = chunk_by_ukey.expect_get(&a);
      let b_chunk = chunk_by_ukey.expect_get(&b);
      let mut queue = a_chunk.groups.iter().copied().collect::<HashSet<_>>();
      for group_ukey in b_chunk.groups.iter() {
        queue.insert(*group_ukey);
      }
      for group_ukey in queue.clone() {
        for modified_chunk_ukey in modified_chunks.clone() {
          if let Some(m_chunk) = get_chunk_from_ukey(&modified_chunk_ukey, chunk_by_ukey) {
            if modified_chunk_ukey != a
              && modified_chunk_ukey != b
              && m_chunk.is_in_group(&group_ukey)
            {
              remaining_chunks_to_merge -= 1;
              if remaining_chunks_to_merge <= 0 {
                break;
              }
              modified_chunks.insert(a);
              modified_chunks.insert(b);
              continue;
            }
          }
        }
        if let Some(group) = get_chunk_group_from_ukey(&group_ukey, chunk_group_by_ukey) {
          for parent in group.parents_iterable() {
            queue.insert(*parent);
          }
        }
      }
    }

    if chunk_graph.can_chunks_be_integrated(&a, &b, chunk_by_ukey, chunk_group_by_ukey) {
      new_chunk_graph.integrate_chunks(
        &a,
        &b,
        &mut new_chunk_by_ukey,
        &mut new_chunk_group_by_ukey,
        &module_graph,
      );
      new_chunk_by_ukey.remove(&b);

      // flag chunk a as modified as further optimization are possible for all children here
      modified_chunks.insert(a);

      remaining_chunks_to_merge -= 1;
      if remaining_chunks_to_merge <= 0 {
        break;
      }

      // Update all affected combinations
      // delete all combination with the removed chunk
      // we will use combinations with the kept chunk instead
      let a_combinations = combinations_by_chunk.get_mut(&a);
      if let Some(a_combinations) = a_combinations {
        for ukey in a_combinations.clone() {
          let combination = combinations.get_mut(&ukey);
          if combination.deleted {
            continue;
          }
          combination.deleted = true;
          combinations.delete(&ukey);
        }
      }

      // Update combinations with the kept chunk with new sizes
      let b_combinations = combinations_by_chunk.get(&b);
      if let Some(b_combinations) = b_combinations {
        for ukey in b_combinations {
          let combination = combinations.get_mut(ukey);
          if combination.deleted {
            continue;
          }
          if combination.a == b {
            if !chunk_graph.can_chunks_be_integrated(&a, &b, chunk_by_ukey, chunk_group_by_ukey) {
              combination.deleted = true;
              combinations.delete(ukey);
              continue;
            }
            // Update size
            let new_integrated_size = chunk_graph.get_integrated_chunks_size(
              &a,
              &combination.b,
              &chunk_size_option,
              chunk_by_ukey,
              chunk_group_by_ukey,
              &module_graph,
              compilation,
            );
            combination.a = a;
            combination.integrated_size = new_integrated_size;
            combination.a_size = integrated_size;
            combination.size_diff = combination.b_size + integrated_size - new_integrated_size;
            combinations.update();
          } else if combination.b == b {
            if !chunk_graph.can_chunks_be_integrated(&b, &a, chunk_by_ukey, chunk_group_by_ukey) {
              combination.deleted = true;
              combinations.delete(ukey);
              continue;
            }
            // Update size
            let new_integrated_size = chunk_graph.get_integrated_chunks_size(
              &combination.a,
              &a,
              &chunk_size_option,
              chunk_by_ukey,
              chunk_group_by_ukey,
              &module_graph,
              compilation,
            );
            combination.b = a;
            combination.integrated_size = new_integrated_size;
            combination.b_size = integrated_size;
            combination.size_diff = integrated_size + combination.a_size - new_integrated_size;
            combinations.update();
          }
        }
      }
      let combinations = combinations_by_chunk
        .get(&b)
        .expect("chunk combination not found");
      combinations_by_chunk.insert(a, combinations.clone());
      combinations_by_chunk.remove(&b);
    }
  }

  compilation.chunk_by_ukey = new_chunk_by_ukey;
  compilation.chunk_group_by_ukey = new_chunk_group_by_ukey;
  compilation.chunk_graph = new_chunk_graph;

  Ok(None)
}

impl Plugin for LimitChunkCountPlugin {
  fn name(&self) -> &'static str {
    "LimitChunkCountPlugin"
  }

  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &mut rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_chunks::new(self));
    Ok(())
  }
}
