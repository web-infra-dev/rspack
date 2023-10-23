mod chunk_combinations;

use std::{
  cmp::Ordering,
  collections::{HashMap, HashSet},
};

use chunk_combinations::{ChunkCombination, ChunkCombinations};
use rspack_core::{
  BoxModule, Chunk, ChunkGraph, ChunkGroup, ChunkUkey, ModuleGraph, OptimizeChunksArgs, Plugin,
  PluginContext, PluginOptimizeChunksOutput,
};
use rspack_database::Database;
use rspack_util::comparators::compare_ids;

// TODO: we should remove this function to crate rspack_util
fn compare_modules_by_identifier(a: &BoxModule, b: &BoxModule) -> std::cmp::Ordering {
  compare_ids(&a.identifier(), &b.identifier())
}

fn compare_module_iterables(modules_a: &[&BoxModule], modules_b: &[&BoxModule]) -> Ordering {
  let mut a_iter = modules_a.iter();
  let mut b_iter = modules_b.iter();
  loop {
    match (a_iter.next(), b_iter.next()) {
      (None, None) => return Ordering::Equal,
      (None, Some(_)) => return Ordering::Less,
      (Some(_), None) => return Ordering::Greater,
      (Some(a_item), Some(b_item)) => {
        let res = compare_modules_by_identifier(&a_item, &b_item);
        if res != Ordering::Equal {
          return res;
        }
      }
    }
  }
}

fn compare_chunks(
  chunk_graph: &ChunkGraph,
  module_graph: &ModuleGraph,
  chunk_a: &ChunkUkey,
  chunk_b: &ChunkUkey,
) -> Ordering {
  let cgc_a = chunk_graph.get_chunk_graph_chunk(chunk_a);
  let cgc_b = chunk_graph.get_chunk_graph_chunk(chunk_b);
  if cgc_a.modules.len() > cgc_b.modules.len() {
    return Ordering::Greater;
  }
  if cgc_a.modules.len() < cgc_b.modules.len() {
    return Ordering::Less;
  }

  let modules_a: Vec<&BoxModule> = cgc_a
    .modules
    .iter()
    .filter_map(|module_id| module_graph.module_by_identifier(module_id))
    .collect();
  let modules_b: Vec<&BoxModule> = cgc_b
    .modules
    .iter()
    .filter_map(|module_id| module_graph.module_by_identifier(module_id))
    .collect();
  compare_module_iterables(&modules_a, &modules_b)
}

// true, if a is always a parent of b
fn is_available_chunk(chunk_group_by_ukey: &Database<ChunkGroup>, a: &Chunk, b: &Chunk) -> bool {
  let mut queue = b.groups.clone();
  while let Some(chunk_group_ukey) = queue.iter().next().cloned() {
    queue.remove(&chunk_group_ukey);
    if a.is_in_group(&chunk_group_ukey) {
      continue;
    }
    let chunk_group = chunk_group_by_ukey.get(&chunk_group_ukey);
    if chunk_group.is_none() {
      continue;
    }
    let chunk_group = chunk_group.unwrap();
    if chunk_group.is_initial() {
      return false;
    }
    for parent in chunk_group.parents_iterable() {
      queue.insert(parent.clone());
    }
  }
  true
}

fn can_chunks_be_integrated(
  chunk_group_by_ukey: &Database<ChunkGroup>,
  chunk_a: &Chunk,
  chunk_b: &Chunk,
) -> bool {
  let has_runtime_a = chunk_a.has_runtime(chunk_group_by_ukey);
  let has_runtime_b = chunk_b.has_runtime(chunk_group_by_ukey);

  if has_runtime_a != has_runtime_b {
    if has_runtime_a {
      return is_available_chunk(chunk_group_by_ukey, chunk_a, chunk_b);
    } else if has_runtime_b {
      return is_available_chunk(chunk_group_by_ukey, chunk_b, chunk_a);
    } else {
      return false;
    }
  }
  true
}

fn get_modules_size(modules: &[&BoxModule]) -> f64 {
  let mut size = 0f64;
  for module in modules {
    for source_type in module.source_types() {
      size += module.size(source_type);
    }
  }
  size
}

struct ChunkSizeOptions {
  // constant overhead for a chunk
  chunk_overhead: Option<f64>,
  // multiplicator for initial chunks
  entry_chunk_multiplicator: Option<f64>,
}

fn get_integrated_chunks_size(
  chunk_graph: &ChunkGraph,
  module_graph: &ModuleGraph,
  chunk_group_by_ukey: &Database<ChunkGroup>,
  chunk_a: &Chunk,
  chunk_b: &Chunk,
  options: &ChunkSizeOptions,
) -> f64 {
  let cgc_a = chunk_graph.get_chunk_graph_chunk(&chunk_a.ukey);
  let cgc_b: &rspack_core::ChunkGraphChunk = chunk_graph.get_chunk_graph_chunk(&chunk_b.ukey);
  let mut all_modules: Vec<&BoxModule> = cgc_a
    .modules
    .iter()
    .filter_map(|id| module_graph.module_by_identifier(id))
    .collect::<Vec<_>>();
  for id in &cgc_b.modules {
    let module = module_graph.module_by_identifier(id);
    if let Some(module) = module {
      all_modules.push(module);
    }
  }
  let modules_size = get_modules_size(&all_modules);
  let chunk_overhead = options.chunk_overhead.unwrap_or(10000f64);
  let entry_chunk_multiplicator = options.entry_chunk_multiplicator.unwrap_or(10f64);
  chunk_overhead
    + modules_size
      * (if chunk_a.can_be_initial(chunk_group_by_ukey)
        || chunk_b.can_be_initial(chunk_group_by_ukey)
      {
        entry_chunk_multiplicator
      } else {
        1f64
      })
}

fn get_chunk_size(
  chunk_graph: &ChunkGraph,
  module_graph: &ModuleGraph,
  chunk_group_by_ukey: &Database<ChunkGroup>,
  chunk: &Chunk,
  options: &ChunkSizeOptions,
) -> f64 {
  let cgc = chunk_graph.get_chunk_graph_chunk(&chunk.ukey);
  let modules: Vec<&BoxModule> = cgc
    .modules
    .iter()
    .filter_map(|id| module_graph.module_by_identifier(id))
    .collect::<Vec<_>>();
  let modules_size = get_modules_size(&modules);
  let chunk_overhead = options.chunk_overhead.unwrap_or(10000f64);
  let entry_chunk_multiplicator = options.entry_chunk_multiplicator.unwrap_or(10f64);
  chunk_overhead
    + modules_size
      * (if chunk.can_be_initial(&chunk_group_by_ukey) {
        entry_chunk_multiplicator
      } else {
        1f64
      })
}

fn add_to_set_map(
  map: &mut HashMap<&ChunkUkey, HashSet<&ChunkCombination>>,
  key: &ChunkUkey,
  value: &ChunkCombination,
) {
  todo!();
}

fn integrate_chunks(chunk_a: &mut Chunk, chunk_b: &Chunk) {
  todo!();
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

#[derive(Debug)]
pub struct LimitChunkCountPlugin {
  options: LimitChunkCountPluginOptions,
}

impl LimitChunkCountPlugin {
  pub fn new(options: LimitChunkCountPluginOptions) -> Self {
    Self { options }
  }
}

#[async_trait::async_trait]
impl Plugin for LimitChunkCountPlugin {
  fn name(&self) -> &'static str {
    "LimitChunkCountPlugin"
  }

  async fn optimize_chunks(
    &self,
    _ctx: PluginContext,
    _args: OptimizeChunksArgs<'_>,
  ) -> PluginOptimizeChunksOutput {
    let compilation = _args.compilation;
    let max_chunks = self.options.max_chunks;
    if max_chunks < 1 {
      return Ok(());
    }

    let chunk_by_ukey = &compilation.chunk_by_ukey;
    let chunk_group_by_ukey = &compilation.chunk_group_by_ukey;

    let mut chunks = compilation.chunk_by_ukey.values().collect::<Vec<_>>();
    if chunks.len() <= max_chunks {
      return Ok(());
    }

    let chunk_graph = &compilation.chunk_graph;
    let module_graph = &compilation.module_graph;
    let mut remaining_chunks_to_merge = chunks.len() - max_chunks;

    // order chunks in a deterministic way
    chunks.sort_by(|a, b| compare_chunks(chunk_graph, module_graph, &a.ukey, &b.ukey));

    // create a lazy sorted data structure to keep all combinations
    // this is large. Size = chunks * (chunks - 1) / 2
    // It uses a multi layer bucket sort plus normal sort in the last layer
    // It's also lazy so only accessed buckets are sorted
    let mut combinations = ChunkCombinations::new();

    // we keep a mapping from chunk to all combinations
    // but this mapping is not kept up-to-date with deletions
    // so `deleted` flag need to be considered when iterating this
    let mut combinations_by_chunk: HashMap<&ChunkUkey, HashSet<&ChunkCombination>> = HashMap::new();

    let chunk_size_option = ChunkSizeOptions {
      chunk_overhead: self.options.chunk_overhead,
      entry_chunk_multiplicator: self.options.entry_chunk_multiplicator,
    };

    for (b_idx, b) in chunks.iter().enumerate() {
      for a_idx in 0..b_idx {
        let a = &chunks[a_idx];
        if !can_chunks_be_integrated(&chunk_group_by_ukey, a, b) {
          continue;
        }

        let integrated_size = get_integrated_chunks_size(
          &chunk_graph,
          &module_graph,
          &chunk_group_by_ukey,
          a,
          b,
          &chunk_size_option,
        );
        let a_size = get_chunk_size(
          &chunk_graph,
          &module_graph,
          &chunk_group_by_ukey,
          a,
          &chunk_size_option,
        );
        let b_size = get_chunk_size(
          &chunk_graph,
          &module_graph,
          &chunk_group_by_ukey,
          b,
          &chunk_size_option,
        );

        let c = ChunkCombination {
          deleted: false,
          size_diff: a_size + b_size - integrated_size,
          integrated_size,
          a: a.ukey,
          b: b.ukey,
          a_idx,
          b_idx,
          a_size,
          b_size,
        };

        add_to_set_map(&mut combinations_by_chunk, &a.ukey, &c);
        add_to_set_map(&mut combinations_by_chunk, &b.ukey, &c);
        combinations.add(c);
      }
    }

    // list of modified chunks during this run
    // combinations affected by this change are skipped to allow
    // further optimizations
    let mut modified_chunks: HashSet<ChunkUkey> = HashSet::new();

    loop {
      let mut combination = match combinations.pop_first() {
        Some(combination) => combination,
        None => break,
      };

      combination.deleted = true;
      let a = combination.a;
      let b = combination.b;
      let integrated_size = combination.integrated_size;

      if !modified_chunks.is_empty() {
        let a_chunk = &chunk_by_ukey.get(&a).unwrap();
        let b_chunk = &chunk_by_ukey.get(&b).unwrap();
        let mut queue = a_chunk
          .groups
          .iter()
          .map(|ukey| ukey.clone())
          .collect::<HashSet<_>>();
        for group_ukey in b_chunk.groups.iter() {
          queue.insert(group_ukey.clone());
        }
        for group_ukey in queue.clone() {
          for modified_chunk_ukey in modified_chunks.clone() {
            let m_chunk = chunk_by_ukey.get(&modified_chunk_ukey);
            if let Some(m_chunk) = m_chunk {
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
          let group = chunk_group_by_ukey.get(&group_ukey);
          if let Some(group) = group {
            for parent in group.parents_iterable() {
              queue.insert(parent.clone());
            }
          }
        }
      }

      // if can_chunks_be_integrated(&chunk_group_by_ukey, &a, b) {
      //   integrate_chunks(&mut combination.a, b);
      //   chunk_by_ukey.remove(&b.ukey);

      //   modified_chunks.insert(a.ukey.clone());

      //   remaining_chunks_to_merge -= 1;
      //   if remaining_chunks_to_merge <= 0 {
      //     break;
      //   }

      //   let mut combinations_a = combinations_by_chunk.get(&a.ukey);
      //   if let Some(combinations_a) = combinations_a.clone() {
      //     for combination in combinations_by_chunk.get(&a.ukey).unwrap() {
      //       if combination.deleted {
      //         continue;
      //       }
      //       combination.deleted = true;
      //       combinations_a.remove(combination);
      //     }
      //   }

      //   let mut combinations_b = combinations_by_chunk.get(&b.ukey);
      //   if let Some(combinations_b) = combinations_b.clone() {
      //     for combination in combinations_b {
      //       if combination.deleted {
      //         continue;
      //       }
      //       if combination.a.ukey == b.ukey {
      //         if !can_chunks_be_integrated(&chunk_group_by_ukey, &a, combination.b) {
      //           combination.deleted = true;
      //           combinations.delete(combination);
      //           continue;
      //         }
      //         let new_integrated_size = get_integrated_chunks_size(
      //           &chunk_graph,
      //           &module_graph,
      //           &chunk_group_by_ukey,
      //           &a,
      //           combination.b,
      //           &chunk_size_option,
      //         );
      //         let finish_update = combinations.start_update(combination);
      //         combination.a = &combination.a;
      //         combination.integrated_size = new_integrated_size;
      //         combination.a_size = integrated_size;
      //         combination.size_diff = combination.b_size + integrated_size - new_integrated_size;
      //         finish_update(true);
      //       } else if combination.b.ukey == b.ukey {
      //         if !can_chunks_be_integrated(&chunk_group_by_ukey, combination.a, &a) {
      //           combination.deleted = true;
      //           combinations.delete(combination);
      //           continue;
      //         }
      //         let new_integrated_size = get_integrated_chunks_size(
      //           &chunk_graph,
      //           &module_graph,
      //           &chunk_group_by_ukey,
      //           combination.a,
      //           &a,
      //           &chunk_size_option,
      //         );
      //         let finish_update = combinations.start_update(combination);
      //         combination.b = &combination.a;
      //         combination.integrated_size = new_integrated_size;
      //         combination.b_size = integrated_size;
      //         combination.size_diff = integrated_size + combination.a_size - new_integrated_size;
      //         finish_update(true);
      //       }
      //     }
      //   }
      //   let combinations = combinations_by_chunk.get(&b.ukey);
      //   if let Some(combinations) = combinations {
      //     combinations_by_chunk.insert(&a.ukey, combinations.clone());
      //   }
      //   combinations_by_chunk.remove(&b.ukey);
      // }
    }

    Ok(())
  }
}
