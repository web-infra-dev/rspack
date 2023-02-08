use std::cmp::Ordering;

use rspack_core::{ChunkByUkey, ChunkGraph, ChunkUkey, Plugin};

use crate::id_helpers::assign_ascending_chunk_ids;

#[derive(Debug)]
pub struct StableNumericChunkIdsPlugin;

impl Plugin for StableNumericChunkIdsPlugin {
  fn chunk_ids(&mut self, compilation: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
    let mut ctx = CompareContext {
      chunk_db: &compilation.chunk_by_ukey,
      chunk_graph: &compilation.chunk_graph,
    };
    let mut chunks = compilation
      .chunk_by_ukey
      .keys()
      .cloned()
      .collect::<Vec<_>>();
    chunks.sort_by(|a, b| compare_chunks_stable(a, b, &mut ctx));
    assign_ascending_chunk_ids(&chunks, compilation);
    Ok(())
  }
}

struct CompareContext<'a> {
  chunk_graph: &'a ChunkGraph,
  chunk_db: &'a ChunkByUkey,
}

macro_rules! return_if_not_equal {
  ($a:expr, $b:expr) => {{
    let res = $a.cmp(&$b);
    if res != Ordering::Equal {
      return res;
    };
  }};
}

fn compare_chunks_stable(a: &ChunkUkey, b: &ChunkUkey, ctx: &mut CompareContext) -> Ordering {
  let a_chunk = a.as_ref(ctx.chunk_db);
  let b_chunk = b.as_ref(ctx.chunk_db);
  if let (Some(a_name), Some(b_name)) = (a_chunk.name.as_ref(), b_chunk.name.as_ref()) {
    return_if_not_equal!(a_name, b_name);
  }

  let mut a_modules = ctx
    .chunk_graph
    .get_chunk_module_identifiers(a)
    .iter()
    .collect::<Vec<_>>();
  a_modules.sort();
  let mut b_modules = ctx
    .chunk_graph
    .get_chunk_module_identifiers(b)
    .iter()
    .collect::<Vec<_>>();
  b_modules.sort();

  return_if_not_equal!(a_modules, b_modules);

  Ordering::Equal
}
