use std::{collections::HashMap, sync::Arc};

use rayon::prelude::*;
use rspack_core::NormalizedBundleOptions;
use rspack_core::PluginDriver;
use rspack_core::{get_swc_compiler, Bundle};
use tracing::instrument;

use self::split_chunks::split_chunks;
pub mod split_chunks;

#[derive(Debug)]
pub struct OutputChunk {
  pub code: String,
  pub file_name: String,
}

#[derive(Debug)]
pub struct ChunkSpliter {
  pub output_options: Arc<NormalizedBundleOptions>,
}

impl ChunkSpliter {
  pub fn new(output_options: Arc<NormalizedBundleOptions>) -> Self {
    Self { output_options }
  }

  #[instrument(skip(self, plugin_dirver, bundle))]
  pub fn generate(
    &mut self,
    plugin_dirver: &PluginDriver,
    bundle: &mut Bundle,
  ) -> HashMap<String, OutputChunk> {
    let mut chunks = split_chunks(&bundle.module_graph, self.output_options.code_splitting);

    chunks.iter_mut().for_each(|chunk| {
      chunk.id = chunk.generate_id(&self.output_options, bundle);
      let entry_module = bundle
        .module_graph
        .module_by_id
        .get_mut(&chunk.entry)
        .unwrap();
      entry_module.add_chunk(chunk.id.clone());
    });

    chunks
      .iter()
      .for_each(|chunk| plugin_dirver.tap_generated_chunk(chunk, &self.output_options));
    let compiler = get_swc_compiler();
    chunks
      .par_iter_mut()
      .map(|chunk| {
        let chunk = chunk.render(&self.output_options, compiler.clone(), bundle);
        (
          chunk.file_name.clone(),
          OutputChunk {
            code: chunk.code,
            file_name: chunk.file_name,
          },
        )
      })
      .collect()
  }
}
