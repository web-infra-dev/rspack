use rspack_core::Plugin;

use crate::id_helpers::{assign_ascending_chunk_ids, compare_chunks_natural};

#[derive(Debug, Default)]
pub struct NaturalChunkIdsPlugin;

impl NaturalChunkIdsPlugin {
  pub fn new() -> Self {
    Self
  }
}

impl Plugin for NaturalChunkIdsPlugin {
  fn name(&self) -> &'static str {
    "rspack.NaturalChunkIdsPlugin"
  }

  fn chunk_ids(&self, compilation: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
    let chunk_graph = &compilation.chunk_graph;
    let module_graph = &compilation.module_graph;
    let mut chunks = compilation.chunk_by_ukey.values_mut().collect::<Vec<_>>();
    chunks.sort_unstable_by(|a, b| compare_chunks_natural(chunk_graph, module_graph, a, b));
    let ukey_chunks = chunks
      .into_iter()
      .map(|chunk| chunk.ukey)
      .collect::<Vec<_>>();
    assign_ascending_chunk_ids(&ukey_chunks, compilation);

    Ok(())
  }
}
