use std::cmp::Ordering;

use rspack_core::{
  BoxModule, ChunkGraph, ChunkUkey, ModuleGraph, OptimizeChunksArgs, Plugin, PluginContext,
  PluginOptimizeChunksOutput,
};
use rspack_util::comparators::compare_ids;

// TODO: we should remove this function to crate rspack_util
pub fn compare_modules_by_identifier(a: &BoxModule, b: &BoxModule) -> std::cmp::Ordering {
  compare_ids(&a.identifier(), &b.identifier())
}

fn compare_module_iterables(modules_a: &Vec<&BoxModule>, modules_b: &Vec<&BoxModule>) -> Ordering {
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

pub fn compare_chunks(
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

#[derive(Debug, Clone, Default)]
pub struct LimitChunkCountPluginOptions {
  max_chunks: usize,
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

    let mut chunks = compilation.chunk_by_ukey.keys().collect::<Vec<_>>();
    if chunks.len() <= max_chunks {
      return Ok(());
    }

    let chunk_graph = &compilation.chunk_graph;
    let module_graph = &compilation.module_graph;
    let remaining_chunks_to_merge = chunks.len() - max_chunks;

    // order chunks in a deterministic way
    chunks.sort_by(|a, b| compare_chunks(chunk_graph, module_graph, a, b));

    Ok(())
  }
}
