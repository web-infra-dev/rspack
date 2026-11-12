mod chunk_combination;

use chunk_combination::{ChunkCombination, ChunkCombinationBucket, ChunkCombinationUkey};
use rspack_core::{
  ChunkSizeOptions, ChunkUkey, Compilation, CompilationOptimizeChunks, Plugin,
  compare_chunks_with_graph, incremental::Mutation,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxHashMap, FxHashSet};

fn add_to_set_map(
  map: &mut FxHashMap<ChunkUkey, FxHashSet<ChunkCombinationUkey>>,
  key: &ChunkUkey,
  value: ChunkCombinationUkey,
) {
  if map.get(key).is_none() {
    let mut set = FxHashSet::default();
    set.insert(value);
    map.insert(*key, set);
  } else {
    let set = map.get_mut(key);
    if let Some(set) = set {
      set.insert(value);
    } else {
      let mut set = FxHashSet::default();
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

struct ChunkSizes {
  // Each module's size per source type, in the order `ChunkGraph::get_chunk_size` adds them.
  module_sizes: Vec<f64>,
  // `module_sizes` added up in that order.
  modules_size: f64,
  can_be_initial: bool,
}

/// The results of `ChunkGraph::get_chunk_size` and `ChunkGraph::get_integrated_chunks_size`,
/// computed from module sizes that are read once per chunk instead of once per pair. The sizes
/// are added in the same order, so the results are the same `f64` values.
struct ChunkSizeCache {
  chunk_overhead: f64,
  entry_chunk_multiplicator: f64,
  sizes: FxHashMap<ChunkUkey, ChunkSizes>,
}

impl ChunkSizeCache {
  fn new(chunks: &[ChunkUkey], options: &ChunkSizeOptions, compilation: &Compilation) -> Self {
    let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
    let chunk_by_ukey = &compilation.build_chunk_graph_artifact.chunk_by_ukey;
    let chunk_group_by_ukey = &compilation.build_chunk_graph_artifact.chunk_group_by_ukey;
    let module_graph = compilation.get_module_graph();
    let sizes = chunks
      .iter()
      .map(|chunk| {
        let mut module_sizes = vec![];
        for module_identifier in chunk_graph.get_chunk_modules_identifier(chunk) {
          if let Some(module) = module_graph.module_by_identifier(module_identifier) {
            for source_type in module.source_types(module_graph) {
              module_sizes.push(module.size(Some(source_type)));
            }
          }
        }
        let modules_size = module_sizes
          .iter()
          .fold(0f64, |size, module_size| size + module_size);
        let can_be_initial = chunk_by_ukey
          .expect_get(chunk)
          .can_be_initial(chunk_group_by_ukey);
        (
          *chunk,
          ChunkSizes {
            module_sizes,
            modules_size,
            can_be_initial,
          },
        )
      })
      .collect();
    Self {
      chunk_overhead: options.chunk_overhead.unwrap_or(10000f64),
      entry_chunk_multiplicator: options.entry_chunk_multiplicator.unwrap_or(10f64),
      sizes,
    }
  }

  fn get(&self, chunk: &ChunkUkey) -> &ChunkSizes {
    self
      .sizes
      .get(chunk)
      .unwrap_or_else(|| panic!("Chunk({chunk:?}) has no size"))
  }

  fn chunk_size(&self, chunk: &ChunkUkey) -> f64 {
    let chunk = self.get(chunk);
    self.chunk_overhead
      + chunk.modules_size
        * (if chunk.can_be_initial {
          self.entry_chunk_multiplicator
        } else {
          1f64
        })
  }

  fn integrated_size(&self, a: &ChunkUkey, b: &ChunkUkey) -> f64 {
    let a = self.get(a);
    let b = self.get(b);
    // Shared modules are counted twice, as in `ChunkGraph::get_integrated_chunks_size`.
    let modules_size = b
      .module_sizes
      .iter()
      .fold(a.modules_size, |size, module_size| size + module_size);
    self.chunk_overhead
      + modules_size
        * (if a.can_be_initial || b.can_be_initial {
          self.entry_chunk_multiplicator
        } else {
          1f64
        })
  }
}

/// Picks the chunk pairs to merge, in merge order. It only reads the chunk graph from before any
/// merge, so the merges can be applied afterwards.
fn plan_merges(
  options: &LimitChunkCountPluginOptions,
  compilation: &Compilation,
  mut chunks_ukeys: Vec<ChunkUkey>,
) -> Vec<(ChunkUkey, ChunkUkey)> {
  let chunk_by_ukey = &compilation.build_chunk_graph_artifact.chunk_by_ukey;
  let chunk_group_by_ukey = &compilation.build_chunk_graph_artifact.chunk_group_by_ukey;
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  let module_graph = compilation.get_module_graph();
  let mut remaining_chunks_to_merge = (chunks_ukeys.len() - options.max_chunks) as i64;

  // order chunks in a deterministic way
  chunks_ukeys.sort_by(|a, b| compare_chunks_with_graph(chunk_graph, module_graph, a, b));

  let sizes = ChunkSizeCache::new(
    &chunks_ukeys,
    &ChunkSizeOptions {
      chunk_overhead: options.chunk_overhead,
      entry_chunk_multiplicator: options.entry_chunk_multiplicator,
    },
    compilation,
  );

  // we keep a mapping from chunk to all combinations
  // but this mapping is not kept up-to-date with deletions
  // so `deleted` flag need to be considered when iterating this
  let mut combinations_by_chunk: FxHashMap<ChunkUkey, FxHashSet<ChunkCombinationUkey>> =
    FxHashMap::default();
  let mut all_combinations = vec![];

  for (b_idx, b) in chunks_ukeys.iter().enumerate() {
    for (a_idx, a) in chunks_ukeys.iter().enumerate().take(b_idx) {
      if !chunk_graph.can_chunks_be_integrated(a, b, chunk_by_ukey, chunk_group_by_ukey) {
        continue;
      }

      let integrated_size = sizes.integrated_size(a, b);
      let a_size = sizes.chunk_size(a);
      let b_size = sizes.chunk_size(b);

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
      all_combinations.push(c);
    }
  }

  // keep all combinations ordered by merge priority
  // this is large. Size = chunks * (chunks - 1) / 2
  let mut combinations = ChunkCombinationBucket::new(all_combinations);

  // list of modified chunks during this run
  // combinations affected by this change are skipped to allow
  // further optimizations
  let mut modified_chunks: FxHashSet<ChunkUkey> = FxHashSet::default();
  let mut merges = vec![];

  while let Some(combination_ukey) = combinations.pop_first() {
    let combination = combinations.get(&combination_ukey);
    let a = combination.a;
    let b = combination.b;
    let integrated_size = combination.integrated_size;

    // skip over pair when
    // one of the already merged chunks is a parent of one of the chunks
    if !modified_chunks.is_empty() {
      let a_chunk = chunk_by_ukey.expect_get(&a);
      let b_chunk = chunk_by_ukey.expect_get(&b);
      let mut queue = a_chunk.groups().iter().copied().collect::<FxHashSet<_>>();
      for group_ukey in b_chunk.groups().iter() {
        queue.insert(*group_ukey);
      }
      for group_ukey in queue.clone() {
        for modified_chunk_ukey in modified_chunks.clone() {
          if let Some(m_chunk) = chunk_by_ukey.get(&modified_chunk_ukey)
            && modified_chunk_ukey != a
            && modified_chunk_ukey != b
            && m_chunk.is_in_group(&group_ukey)
          {
            remaining_chunks_to_merge -= 1;
            if remaining_chunks_to_merge <= 0 {
              break;
            }
            modified_chunks.insert(a);
            modified_chunks.insert(b);
          }
        }
        if let Some(group) = chunk_group_by_ukey.get(&group_ukey) {
          for parent in group.parents_iterable() {
            queue.insert(*parent);
          }
        }
      }
    }

    if chunk_graph.can_chunks_be_integrated(&a, &b, chunk_by_ukey, chunk_group_by_ukey) {
      merges.push((a, b));

      // flag chunk a as modified as further optimization are possible for all children here
      modified_chunks.insert(a);

      remaining_chunks_to_merge -= 1;
      if remaining_chunks_to_merge <= 0 {
        break;
      }

      // Update all affected combinations
      // delete all combination with the removed chunk
      // we will use combinations with the kept chunk instead
      if let Some(a_combinations) = combinations_by_chunk.get(&a) {
        for ukey in a_combinations {
          if !combinations.get(ukey).deleted {
            combinations.delete(ukey);
          }
        }
      }

      // The integrability checks below only ever look at the merged pair, so they are computed once
      // per merge. (a, b) has just passed above, so only (b, a) is computed.
      let can_integrate_b_a =
        chunk_graph.can_chunks_be_integrated(&b, &a, chunk_by_ukey, chunk_group_by_ukey);

      // Update combinations with the kept chunk with new sizes
      if let Some(b_combinations) = combinations_by_chunk.get(&b) {
        for ukey in b_combinations {
          let combination = combinations.get(ukey);
          if combination.deleted {
            continue;
          }
          let combination_a = combination.a;
          let combination_b = combination.b;
          if combination_a == b {
            // Update size
            let new_integrated_size = sizes.integrated_size(&a, &combination_b);
            combinations.update(ukey, |combination| {
              combination.a = a;
              combination.integrated_size = new_integrated_size;
              combination.a_size = integrated_size;
              combination.size_diff = combination.b_size + integrated_size - new_integrated_size;
            });
          } else if combination_b == b {
            if !can_integrate_b_a {
              combinations.delete(ukey);
              continue;
            }
            // Update size
            let new_integrated_size = sizes.integrated_size(&combination_a, &a);
            combinations.update(ukey, |combination| {
              combination.b = a;
              combination.integrated_size = new_integrated_size;
              combination.b_size = integrated_size;
              combination.size_diff = integrated_size + combination.a_size - new_integrated_size;
            });
          }
        }
      }
      let b_combinations = combinations_by_chunk
        .remove(&b)
        .expect("chunk combination not found");
      combinations_by_chunk.insert(a, b_combinations);
    }
  }

  merges
}

#[plugin_hook(CompilationOptimizeChunks for LimitChunkCountPlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_ADVANCED)]
async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let max_chunks = self.options.max_chunks;
  if max_chunks < 1 {
    return Ok(None);
  }

  let chunks_ukeys = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .keys()
    .copied()
    .collect::<Vec<_>>();
  if chunks_ukeys.len() <= max_chunks {
    return Ok(None);
  }

  let merges = plan_merges(&self.options, compilation, chunks_ukeys);

  let mut chunk_by_ukey = std::mem::take(&mut compilation.build_chunk_graph_artifact.chunk_by_ukey);
  let mut chunk_group_by_ukey =
    std::mem::take(&mut compilation.build_chunk_graph_artifact.chunk_group_by_ukey);
  let mut chunk_graph = std::mem::take(&mut compilation.build_chunk_graph_artifact.chunk_graph);
  let mut named_chunks = std::mem::take(&mut compilation.build_chunk_graph_artifact.named_chunks);
  let module_graph = compilation.get_module_graph();

  let mut removed_chunks: FxHashSet<ChunkUkey> = FxHashSet::default();
  let mut integrated_chunks: FxHashSet<ChunkUkey> = FxHashSet::default();
  for (a, b) in merges {
    chunk_graph.integrate_chunks(
      &a,
      &b,
      &mut chunk_by_ukey,
      &mut chunk_group_by_ukey,
      &mut named_chunks,
      module_graph,
    );
    integrated_chunks.insert(a);
    chunk_by_ukey.remove(&b);
    removed_chunks.insert(b);
  }

  compilation.build_chunk_graph_artifact.chunk_by_ukey = chunk_by_ukey;
  compilation.build_chunk_graph_artifact.chunk_group_by_ukey = chunk_group_by_ukey;
  compilation.build_chunk_graph_artifact.chunk_graph = chunk_graph;
  compilation.build_chunk_graph_artifact.named_chunks = named_chunks;

  if let Some(mut mutations) = compilation.incremental.mutations_write() {
    // ChunkRemove mutations must be added last because a chunk can be removed after another chunk
    // has been integrated into it
    for chunk in integrated_chunks {
      mutations.add(Mutation::ChunksIntegrate { to: chunk });
    }
    for chunk in removed_chunks {
      mutations.add(Mutation::ChunkRemove { chunk });
    }
  }

  Ok(None)
}

impl Plugin for LimitChunkCountPlugin {
  fn name(&self) -> &'static str {
    "LimitChunkCountPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_chunks::new(self));
    Ok(())
  }
}
