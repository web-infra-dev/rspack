use rspack_core::Bundle;
use rspack_core::ChunkGraph;

use self::split_chunks::split_chunks;
pub mod split_chunks;

#[derive(Debug, Clone)]
pub struct OutputChunk {
  pub code: String,
  pub file_name: String,
  pub entry: String,
}

pub fn generate_chunks(bundle: &mut Bundle) -> ChunkGraph {
  let mut chunk_graph = split_chunks(&bundle.module_graph_container, &bundle.context.options);

  chunk_graph.chunks_mut().for_each(|chunk| {
    let filename = chunk.generate_filename(&bundle.context.options, bundle);
    let entry_module = bundle
      .module_graph_container
      .module_graph
      .module_by_uri_mut(&chunk.entry_uri)
      .unwrap();
    chunk.filename = Some(filename.clone());
    entry_module.add_chunk(filename);
  });

  // TODO: we could do bundle splitting here

  chunk_graph.chunks().for_each(|chunk| {
    bundle
      .plugin_driver
      .tap_generated_chunk(chunk, &bundle.context.options);
  });

  chunk_graph
}
