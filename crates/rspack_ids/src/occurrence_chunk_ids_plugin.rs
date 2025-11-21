use std::collections::HashMap;

use itertools::Itertools;
use rspack_collections::DatabaseItem;
use rspack_core::{Chunk, CompilationChunkIds, Plugin, incremental::IncrementalPasses};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::id_helpers::{assign_ascending_chunk_ids, compare_chunks_natural};

#[derive(Debug)]
pub struct OccurrenceChunkIdsPluginOptions {
  pub prioritise_initial: bool,
}

#[plugin]
#[derive(Debug, Default)]
pub struct OccurrenceChunkIdsPlugin {
  prioritise_initial: bool,
}

impl OccurrenceChunkIdsPlugin {
  pub fn new(option: OccurrenceChunkIdsPluginOptions) -> Self {
    Self::new_inner(option.prioritise_initial)
  }
}

#[plugin_hook(CompilationChunkIds for OccurrenceChunkIdsPlugin)]
async fn chunk_ids(&self, compilation: &mut rspack_core::Compilation) -> Result<()> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::CHUNK_IDS,
    "OccurrenceChunkIdsPlugin (optimization.chunkIds = \"size\")",
    "it requires calculating the id of all the chunks, which is a global effect",
  ) {
    if let Some(diagnostic) = diagnostic {
      compilation.push_diagnostic(diagnostic);
    }
    compilation.chunk_ids_artifact.clear();
  }

  let chunk_graph = &compilation.chunk_graph;
  let chunk_group_by_ukey = &compilation.chunk_group_by_ukey;
  let mut occurs_in_initial_chunks_map = HashMap::new();

  for chunk in compilation.chunk_by_ukey.values() {
    let mut occurs = 0;
    for chunk_group_ukey in chunk.groups() {
      if let Some(chunk_group) = chunk_group_by_ukey.get(chunk_group_ukey) {
        for parent_ukey in &chunk_group.parents {
          if let Some(parent) = chunk_group_by_ukey.get(parent_ukey)
            && parent.is_initial()
          {
            occurs += 1;
          }
        }
      }
    }
    occurs_in_initial_chunks_map.insert(chunk.ukey(), occurs);
  }

  let mut ordered_chunk_modules_cache = Default::default();
  let chunks = compilation
    .chunk_by_ukey
    .values()
    .map(|chunk| chunk as &Chunk)
    .sorted_unstable_by(|a, b| {
      if self.prioritise_initial {
        let a_entry_occurs = occurs_in_initial_chunks_map.get(&a.ukey()).unwrap_or(&0);
        let b_entry_occurs = occurs_in_initial_chunks_map.get(&b.ukey()).unwrap_or(&0);
        if a_entry_occurs != b_entry_occurs {
          return b_entry_occurs.cmp(a_entry_occurs);
        }
      }

      let a_occurs = a.get_number_of_groups();
      let b_occurs = b.get_number_of_groups();
      if a_occurs != b_occurs {
        return b_occurs.cmp(&a_occurs);
      }

      compare_chunks_natural(
        chunk_graph,
        &compilation.chunk_group_by_ukey,
        &compilation.module_ids_artifact,
        a,
        b,
        &mut ordered_chunk_modules_cache,
      )
    })
    .map(|chunk| chunk.ukey())
    .collect::<Vec<_>>();

  assign_ascending_chunk_ids(&chunks, compilation);

  Ok(())
}

impl Plugin for OccurrenceChunkIdsPlugin {
  fn name(&self) -> &'static str {
    "rspack.OccurrenceChunkIdsPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compilation_hooks.chunk_ids.tap(chunk_ids::new(self));
    Ok(())
  }
}
