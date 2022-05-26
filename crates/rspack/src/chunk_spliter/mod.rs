use std::{collections::HashMap, sync::Arc};

use dashmap::DashMap;
use rayon::prelude::*;
use rspack_core::NormalizedBundleOptions;
use rspack_core::PluginDriver;
use rspack_core::{get_swc_compiler, Bundle};
use rspack_swc::swc::TransformOutput;
use tracing::instrument;

use self::split_chunks::split_chunks;
pub mod split_chunks;

#[derive(Debug, Clone)]
pub struct OutputChunk {
  pub code: String,
  pub file_name: String,
  pub entry: String,
}

#[derive(Debug)]
pub struct ChunkSpliter {
  pub output_options: Arc<NormalizedBundleOptions>,
  pub output_modules: DashMap<String, Arc<TransformOutput>>,
}

impl ChunkSpliter {
  pub fn new(output_options: Arc<NormalizedBundleOptions>) -> Self {
    Self {
      output_options,
      output_modules: DashMap::default(),
    }
  }

  #[instrument(skip(self, plugin_driver, bundle))]
  pub fn generate(
    &mut self,
    plugin_driver: &PluginDriver,
    bundle: &mut Bundle,
  ) -> HashMap<String, OutputChunk> {
    let mut chunks = split_chunks(&bundle.module_graph, &self.output_options);

    chunks.iter_mut().for_each(|chunk| {
      let filename = chunk.generate_filename(&self.output_options, bundle);
      let entry_module = bundle
        .module_graph
        .module_graph
        .module_by_uri_mut(&chunk.entry_uri)
        .unwrap();
      chunk.filename = Some(filename.clone());
      entry_module.add_chunk(filename);
    });

    chunks
      .iter()
      .for_each(|chunk| plugin_driver.tap_generated_chunk(chunk, &self.output_options));
    let compiler = get_swc_compiler();
    chunks
      .par_iter_mut()
      .map(|chunk| {
        let chunk = chunk.render(
          &self.output_options,
          compiler.clone(),
          bundle,
          &self.output_modules,
        );
        (
          chunk.file_name.clone(),
          OutputChunk {
            code: chunk.code,
            file_name: chunk.file_name,
            entry: chunk.entry,
          },
        )
      })
      .collect()
  }
}
