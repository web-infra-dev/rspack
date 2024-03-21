#![feature(option_get_or_insert_default)]
#![feature(let_chains)]

use async_trait::async_trait;
use rspack_core::{
  is_runtime_equal, ChunkUkey, OptimizeChunksArgs, Plugin, PluginContext,
  PluginOptimizeChunksOutput,
};
use rustc_hash::FxHashSet as HashSet;

#[derive(Debug)]
pub struct MergeDuplicateChunksPlugin;

#[async_trait]
impl Plugin for MergeDuplicateChunksPlugin {
  fn name(&self) -> &'static str {
    "rspack.MergeDuplicateChunksPlugin"
  }

  async fn optimize_chunks(
    &self,
    _ctx: PluginContext,
    args: OptimizeChunksArgs<'_>,
  ) -> PluginOptimizeChunksOutput {
    let compilation = args.compilation;

    let mut not_duplicates = HashSet::default();

    let mut chunk_ukeys = compilation
      .chunk_by_ukey
      .keys()
      .copied()
      .collect::<Vec<_>>();

    chunk_ukeys.sort_by_key(|ukey| compilation.chunk_by_ukey.expect_get(ukey).name.as_ref());

    for chunk_ukey in chunk_ukeys {
      if !compilation.chunk_by_ukey.contains(&chunk_ukey) {
        // already remove by duplicates
        continue;
      }
      let mut possible_duplicates: Option<HashSet<ChunkUkey>> = None;
      for module in compilation
        .chunk_graph
        .get_chunk_modules(&chunk_ukey, &compilation.get_module_graph())
      {
        if let Some(ref mut possible_duplicates) = possible_duplicates {
          possible_duplicates.retain(|dup| {
            compilation
              .chunk_graph
              .is_module_in_chunk(&module.identifier(), *dup)
          });
          if possible_duplicates.is_empty() {
            break;
          }
        } else {
          for dup in compilation
            .chunk_graph
            .get_module_chunks(module.identifier())
          {
            if *dup != chunk_ukey
              && compilation
                .chunk_graph
                .get_number_of_chunk_modules(&chunk_ukey)
                == compilation.chunk_graph.get_number_of_chunk_modules(dup)
              && !not_duplicates.contains(dup)
            {
              possible_duplicates.get_or_insert_default().insert(*dup);
            }
          }
          if possible_duplicates.is_none() {
            break;
          }
        }
      }

      if let Some(possible_duplicates) = possible_duplicates
        && !possible_duplicates.is_empty()
      {
        'outer: for other_chunk_ukey in possible_duplicates {
          let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
          let other_chunk = compilation.chunk_by_ukey.expect_get(&other_chunk_ukey);
          if other_chunk.has_runtime(&compilation.chunk_group_by_ukey)
            != chunk.has_runtime(&compilation.chunk_group_by_ukey)
          {
            continue;
          }
          if compilation
            .chunk_graph
            .get_number_of_entry_modules(&chunk_ukey)
            > 0
          {
            continue;
          }
          if compilation
            .chunk_graph
            .get_number_of_entry_modules(&other_chunk_ukey)
            > 0
          {
            continue;
          }
          if !is_runtime_equal(&chunk.runtime, &other_chunk.runtime) {
            let module_graph = compilation.get_module_graph();
            for module in compilation
              .chunk_graph
              .get_chunk_modules(&chunk_ukey, &compilation.get_module_graph())
            {
              let exports_info = module_graph.get_exports_info(&module.identifier());
              if !exports_info.is_equally_used(&chunk.runtime, &other_chunk.runtime, &module_graph)
              {
                continue 'outer;
              }
            }
          }
          if compilation.chunk_graph.can_chunks_be_integrated(
            &chunk_ukey,
            &other_chunk_ukey,
            &compilation.chunk_by_ukey,
            &compilation.chunk_group_by_ukey,
          ) {
            let mut chunk_graph = std::mem::take(&mut compilation.chunk_graph);
            let mut chunk_by_ukey = std::mem::take(&mut compilation.chunk_by_ukey);
            let mut chunk_group_by_ukey = std::mem::take(&mut compilation.chunk_group_by_ukey);
            chunk_graph.integrate_chunks(
              &chunk_ukey,
              &other_chunk_ukey,
              &mut chunk_by_ukey,
              &mut chunk_group_by_ukey,
              &compilation.get_module_graph(),
            );
            chunk_by_ukey.remove(&other_chunk_ukey);
            compilation.chunk_graph = chunk_graph;
            compilation.chunk_by_ukey = chunk_by_ukey;
            compilation.chunk_group_by_ukey = chunk_group_by_ukey;
          }
        }
      }

      not_duplicates.insert(chunk_ukey);
    }
    Ok(())
  }
}
