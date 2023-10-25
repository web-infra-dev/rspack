mod chunk_combination;

use std::{
  cmp::Ordering,
  collections::{HashMap, HashSet},
};

use chunk_combination::{ChunkCombination, ChunkCombinationBucket, ChunkCombinationUkey};
use rspack_core::{
  BoxModule, Chunk, ChunkGraph, ChunkGroup, ChunkSizeOptions, ChunkUkey, ModuleGraph,
  OptimizeChunksArgs, Plugin, PluginContext, PluginOptimizeChunksOutput, RuntimeSpec,
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
      (None, Some(_)) => return Ordering::Greater,
      (Some(_), None) => return Ordering::Less,
      (Some(a_item), Some(b_item)) => {
        let res = compare_modules_by_identifier(&a_item, &b_item);
        if res != Ordering::Equal {
          return res;
        }
      }
    }
  }
}

fn compare_chunks_with_graph(
  chunk_graph: &ChunkGraph,
  module_graph: &ModuleGraph,
  chunk_a_ukey: &ChunkUkey,
  chunk_b_ukey: &ChunkUkey,
) -> Ordering {
  let cgc_a = chunk_graph.get_chunk_graph_chunk(chunk_a_ukey);
  let cgc_b = chunk_graph.get_chunk_graph_chunk(chunk_b_ukey);
  if cgc_a.modules.len() > cgc_b.modules.len() {
    return Ordering::Less;
  }
  if cgc_a.modules.len() < cgc_b.modules.len() {
    return Ordering::Greater;
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

fn add_to_set_map(
  map: &mut HashMap<ChunkUkey, HashSet<ChunkCombinationUkey>>,
  key: &ChunkUkey,
  value: ChunkCombinationUkey,
) {
  if map.get(key).is_none() {
    let mut set = HashSet::new();
    set.insert(value);
    map.insert(key.clone(), set);
  } else {
    let set = map.get_mut(key);
    if let Some(set) = set {
      set.insert(value);
    } else {
      let mut set = HashSet::new();
      set.insert(value);
      map.insert(key.clone(), set);
    }
  }
}

fn merge_runtime(a: &RuntimeSpec, b: &RuntimeSpec) -> RuntimeSpec {
  let mut set: RuntimeSpec = Default::default();
  for r in a {
    set.insert(r.clone());
  }
  for r in b {
    set.insert(r.clone());
  }
  set
}

fn integrate_chunks(
  chunk_graph: &mut ChunkGraph,
  module_graph: &ModuleGraph,
  chunk_by_ukey: &mut Database<Chunk>,
  chunk_group_by_ukey: &mut Database<ChunkGroup>,
  a: &ChunkUkey,
  b: &ChunkUkey,
) {
  let chunk_b = chunk_by_ukey.expect_get(b).clone();
  let chunk_a = chunk_by_ukey.expect_mut(a);

  // Decide for one name (deterministic)
  if let (Some(_), Some(_)) = (&chunk_a.name, &chunk_b.name) {
    if (chunk_graph.get_number_of_entry_modules(a) > 0)
      == (chunk_graph.get_number_of_entry_modules(b) > 0)
    {
      // When both chunks have entry modules or none have one, use
      // shortest name
      if chunk_a.name.as_ref().unwrap().len() != chunk_b.name.as_ref().unwrap().len() {
        chunk_a.name =
          if chunk_a.name.as_ref().unwrap().len() < chunk_b.name.as_ref().unwrap().len() {
            chunk_a.name.clone()
          } else {
            chunk_b.name.clone()
          };
      } else {
        chunk_a.name = if chunk_a.name.as_ref().unwrap() < chunk_b.name.as_ref().unwrap() {
          chunk_a.name.clone()
        } else {
          chunk_b.name.clone()
        };
      }
    } else if chunk_graph.get_number_of_entry_modules(b) > 0 {
      // Pick the name of the chunk with the entry module
      chunk_a.name = chunk_b.name.clone();
    }
  } else if chunk_b.name.is_some() {
    chunk_a.name = chunk_b.name.clone();
  }

  // Merge id name hints
  for hint in &chunk_b.id_name_hints {
    chunk_a.id_name_hints.insert(hint.clone());
  }

  // Merge runtime
  chunk_a.runtime = merge_runtime(&chunk_a.runtime, &chunk_b.runtime);

  // get_chunk_modules is used here to create a clone, because disconnect_chunk_and_module modifies
  for module in chunk_graph.get_chunk_modules(b, module_graph) {
    chunk_graph.disconnect_chunk_and_module(b, module.identifier());
    chunk_graph.connect_chunk_and_module(a.clone(), module.identifier());
  }

  for (module, chunk_group) in chunk_graph
    .clone()
    .get_chunk_entry_modules_with_chunk_group_iterable(b)
    .iter()
  {
    chunk_graph.disconnect_chunk_and_entry_module(b, module.clone());
    chunk_graph.connect_chunk_and_entry_module(a.clone(), module.clone(), chunk_group.clone());
  }

  let mut remove_group_ukeys = vec![];
  for chunk_group_ukey in chunk_b.groups {
    let chunk_group = chunk_group_by_ukey.expect_mut(&chunk_group_ukey);
    chunk_group.replace_chunk(b, a);
    chunk_a.add_group(chunk_group_ukey);
    remove_group_ukeys.push(chunk_group_ukey);
  }

  let chunk_b = chunk_by_ukey.expect_mut(b);
  for group_ukey in remove_group_ukeys {
    chunk_b.remove_group(&group_ukey);
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

    let chunk_by_ukey = compilation.chunk_by_ukey.clone();
    let chunk_group_by_ukey = compilation.chunk_group_by_ukey.clone();

    let mut chunks_ukeys = compilation
      .chunk_by_ukey
      .keys()
      .map(|key| key.clone())
      .collect::<Vec<_>>();
    if chunks_ukeys.len() <= max_chunks {
      return Ok(());
    }

    let chunk_graph = &compilation.chunk_graph.clone();
    let module_graph = &compilation.module_graph;
    let mut remaining_chunks_to_merge = chunks_ukeys.len() - max_chunks;

    // order chunks in a deterministic way
    chunks_ukeys.sort_by(|a, b| compare_chunks_with_graph(chunk_graph, module_graph, a, b));

    // create a lazy sorted data structure to keep all combinations
    // this is large. Size = chunks * (chunks - 1) / 2
    // It uses a multi layer bucket sort plus normal sort in the last layer
    // It's also lazy so only accessed buckets are sorted
    let mut combinations = ChunkCombinationBucket::new();

    // we keep a mapping from chunk to all combinations
    // but this mapping is not kept up-to-date with deletions
    // so `deleted` flag need to be considered when iterating this
    let mut combinations_by_chunk: HashMap<ChunkUkey, HashSet<ChunkCombinationUkey>> =
      HashMap::new();

    let chunk_size_option = ChunkSizeOptions {
      chunk_overhead: self.options.chunk_overhead,
      entry_chunk_multiplicator: self.options.entry_chunk_multiplicator,
    };

    for (b_idx, b) in chunks_ukeys.iter().enumerate() {
      for a_idx in 0..b_idx {
        let a = &chunks_ukeys[a_idx];
        if !chunk_graph.can_chunks_be_integrated(&a, &b, &chunk_by_ukey, &chunk_group_by_ukey) {
          continue;
        }

        let integrated_size = chunk_graph.get_integrated_chunks_size(
          &a,
          &b,
          &chunk_size_option,
          &chunk_by_ukey,
          &chunk_group_by_ukey,
          &module_graph,
        );
        let a_size = chunk_graph.get_chunk_size(
          a,
          &chunk_size_option,
          &chunk_by_ukey,
          &chunk_group_by_ukey,
          &module_graph,
        );
        let b_size = chunk_graph.get_chunk_size(
          b,
          &chunk_size_option,
          &chunk_by_ukey,
          &chunk_group_by_ukey,
          &module_graph,
        );

        let c = ChunkCombination {
          ukey: ChunkCombinationUkey::new(),
          deleted: false,
          size_diff: a_size + b_size - integrated_size,
          integrated_size,
          a: a.clone(),
          b: b.clone(),
          a_idx,
          b_idx,
          a_size,
          b_size,
        };

        add_to_set_map(&mut combinations_by_chunk, &a, c.ukey);
        add_to_set_map(&mut combinations_by_chunk, &b, c.ukey);
        combinations.add(c);
      }
    }

    // list of modified chunks during this run
    // combinations affected by this change are skipped to allow
    // further optimizations
    let mut modified_chunks: HashSet<ChunkUkey> = HashSet::new();

    loop {
      let combination_ukey = match combinations.pop_first() {
        Some(combination_ukey) => combination_ukey,
        None => break,
      };

      let combination = combinations.get_mut(&combination_ukey).unwrap();
      combination.deleted = true;
      let a = combination.a;
      let b = combination.b;
      let integrated_size = combination.integrated_size;

      // skip over pair when
      // one of the already merged chunks is a parent of one of the chunks
      if !modified_chunks.is_empty() {
        let a_chunk = chunk_by_ukey.get(&a).unwrap();
        let b_chunk = chunk_by_ukey.get(&b).unwrap();
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

      if chunk_graph.can_chunks_be_integrated(&a, &b, &chunk_by_ukey, &chunk_group_by_ukey) {
        integrate_chunks(
          &mut compilation.chunk_graph,
          &module_graph,
          &mut compilation.chunk_by_ukey,
          &mut compilation.chunk_group_by_ukey,
          &a,
          &b,
        );
        compilation.chunk_by_ukey.remove(&b);

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
            let combination = combinations.get_mut(&ukey).unwrap();
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
            let combination = combinations.get_mut(ukey).unwrap();
            if combination.deleted {
              continue;
            }
            if combination.a == b {
              if !chunk_graph.can_chunks_be_integrated(&a, &b, &chunk_by_ukey, &chunk_group_by_ukey)
              {
                combination.deleted = true;
                combinations.delete(ukey);
                continue;
              }
              // Update size
              let new_integrated_size = chunk_graph.get_integrated_chunks_size(
                &a,
                &combination.b,
                &chunk_size_option,
                &chunk_by_ukey,
                &chunk_group_by_ukey,
                &module_graph,
              );
              combination.a = a;
              combination.integrated_size = new_integrated_size;
              combination.a_size = integrated_size;
              combination.size_diff = combination.b_size + integrated_size - new_integrated_size;
              combinations.update();
            } else if combination.b == b {
              if !chunk_graph.can_chunks_be_integrated(&b, &a, &chunk_by_ukey, &chunk_group_by_ukey)
              {
                combination.deleted = true;
                combinations.delete(ukey);
                continue;
              }
              // Update size
              let new_integrated_size = chunk_graph.get_integrated_chunks_size(
                &combination.a,
                &a,
                &chunk_size_option,
                &chunk_by_ukey,
                &chunk_group_by_ukey,
                &module_graph,
              );
              combination.b = a;
              combination.integrated_size = new_integrated_size;
              combination.b_size = integrated_size;
              combination.size_diff = integrated_size + combination.a_size - new_integrated_size;
              combinations.update();
            }
          }
        }
        let combinations = combinations_by_chunk.get(&b).unwrap();
        combinations_by_chunk.insert(a.clone(), combinations.clone());
        combinations_by_chunk.remove(&b);
      }
    }

    Ok(())
  }
}
