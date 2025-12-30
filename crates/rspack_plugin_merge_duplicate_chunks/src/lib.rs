use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rspack_collections::UkeySet;
use rspack_core::{
  ChunkUkey, Compilation, CompilationOptimizeChunks, ExportsInfo, ModuleGraph, Plugin, RuntimeSpec,
  incremental::Mutation, is_runtime_equal,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashSet as HashSet;

#[plugin]
#[derive(Debug, Default)]
pub struct MergeDuplicateChunksPlugin;

#[plugin_hook(CompilationOptimizeChunks for MergeDuplicateChunksPlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_BASIC)]
async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let mut not_duplicates = HashSet::default();

  let mut chunk_ukeys = compilation
    .chunk_by_ukey
    .keys()
    .copied()
    .collect::<Vec<_>>();

  chunk_ukeys.sort_by_key(|ukey| compilation.chunk_by_ukey.expect_get(ukey).name());

  for chunk_ukey in chunk_ukeys {
    if !compilation.chunk_by_ukey.contains(&chunk_ukey) {
      // already remove by duplicates
      continue;
    }
    let mut possible_duplicates: Option<UkeySet<ChunkUkey>> = None;
    for module in compilation
      .chunk_graph
      .get_chunk_modules_identifier(&chunk_ukey)
    {
      if let Some(ref mut possible_duplicates) = possible_duplicates {
        possible_duplicates.retain(|dup| compilation.chunk_graph.is_module_in_chunk(module, *dup));
        if possible_duplicates.is_empty() {
          break;
        }
      } else {
        for dup in compilation.chunk_graph.get_module_chunks(*module) {
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
        if !is_runtime_equal(chunk.runtime(), other_chunk.runtime()) {
          let module_graph = compilation.get_module_graph();
          let is_all_equal = compilation
            .chunk_graph
            .get_chunk_modules_identifier(&chunk_ukey)
            .into_par_iter()
            .all(|module| {
              let exports_info = module_graph.get_exports_info(module);
              is_equally_used(
                &exports_info,
                module_graph,
                chunk.runtime(),
                other_chunk.runtime(),
              )
            });
          if !is_all_equal {
            continue 'outer;
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
            compilation.get_module_graph(),
          );
          if chunk_by_ukey.remove(&other_chunk_ukey).is_some()
            && let Some(mut mutations) = compilation.incremental.mutations_write()
          {
            mutations.add(Mutation::ChunksIntegrate { to: chunk_ukey });
            mutations.add(Mutation::ChunkRemove {
              chunk: other_chunk_ukey,
            });
          }
          compilation.chunk_graph = chunk_graph;
          compilation.chunk_by_ukey = chunk_by_ukey;
          compilation.chunk_group_by_ukey = chunk_group_by_ukey;
        }
      }
    }

    not_duplicates.insert(chunk_ukey);
  }
  Ok(None)
}

impl Plugin for MergeDuplicateChunksPlugin {
  fn name(&self) -> &'static str {
    "rspack.MergeDuplicateChunksPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext) -> Result<()> {
    ctx
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_chunks::new(self));
    Ok(())
  }
}

fn is_equally_used(
  exports_info: &ExportsInfo,
  mg: &ModuleGraph,
  a: &RuntimeSpec,
  b: &RuntimeSpec,
) -> bool {
  let info = exports_info.as_data(mg);
  let other_exports_info = info.other_exports_info();
  if other_exports_info.get_used(Some(a)) != other_exports_info.get_used(Some(b)) {
    return false;
  }
  let side_effects_only_info = info.side_effects_only_info();
  if side_effects_only_info.get_used(Some(a)) != side_effects_only_info.get_used(Some(b)) {
    return false;
  }
  for export_info in info.exports().values() {
    let export_info_data = export_info;
    if export_info_data.get_used(Some(a)) != export_info_data.get_used(Some(b)) {
      return false;
    }
  }
  true
}
