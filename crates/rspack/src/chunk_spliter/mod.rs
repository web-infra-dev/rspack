use rspack_core::Bundle;
use rspack_core::ChunkGraph;

use self::split_chunks::split_chunks;
pub mod split_chunks;

pub fn generate_chunks(bundle: &mut Bundle) -> ChunkGraph {
  let mut chunk_graph = split_chunks(&bundle.module_graph_container, &bundle.context.options);

  chunk_graph.chunks_mut().for_each(|chunk| {
    let filename = chunk.generate_filename(&bundle.context.options, bundle);
    chunk.filename = Some(filename);
  });

  // TODO: we could do bundle splitting here

  chunk_graph.chunks().for_each(|chunk| {
    bundle
      .plugin_driver
      .tap_generated_chunk(chunk, &bundle.context.options);
  });

  chunk_graph
}
