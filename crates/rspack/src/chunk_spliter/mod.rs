use rayon::prelude::*;
use rspack_core::BundleOptions;
use rspack_core::Chunk;

use rspack_core::Bundle;
use rspack_core::PluginDriver;

use self::split_chunks::split_chunks;
pub mod split_chunks;

#[derive(Debug, Clone)]
pub struct OutputChunk {
  pub code: String,
  pub file_name: String,
  pub entry: String,
}

pub fn generate_chunks(
  output_options: &BundleOptions,
  plugin_driver: &PluginDriver,
  bundle: &mut Bundle,
) -> Vec<Chunk> {
  let mut chunks = split_chunks(&bundle.module_graph_container, output_options);

  chunks.iter_mut().for_each(|chunk| {
    let filename = chunk.generate_filename(output_options, bundle);
    let entry_module = bundle
      .module_graph_container
      .module_graph
      .module_by_uri_mut(&chunk.entry_uri)
      .unwrap();
    chunk.filename = Some(filename.clone());
    entry_module.add_chunk(filename);
  });

  // TODO: we could do bundle splitting here

  chunks
    .iter()
    .for_each(|chunk| plugin_driver.tap_generated_chunk(chunk, output_options));
  chunks
}
