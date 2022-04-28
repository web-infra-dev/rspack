use std::{collections::HashMap, sync::Arc};

use crate::{
  bundler::BundleOptions, module_graph::ModuleGraph, plugin_driver::PluginDriver,
  structs::OutputChunk,
};
use tracing::instrument;

use self::split_chunks::split_chunks;
pub mod split_chunks;

#[derive(Debug)]
pub struct Bundle {
  pub graph: ModuleGraph,
  pub output_options: Arc<BundleOptions>,
}

impl Bundle {
  pub fn new(graph: ModuleGraph, output_options: Arc<BundleOptions>) -> Self {
    Self {
      graph,
      output_options,
    }
  }

  #[instrument(skip(self))]
  pub fn generate(&mut self, plugin_dirver: &PluginDriver) -> HashMap<String, OutputChunk> {
    let mut chunks = split_chunks(&self.graph);

    chunks.iter_mut().for_each(|chunk| {
      chunk.id = chunk.generate_id(&self.output_options);
    });

    chunks
      .iter()
      .for_each(|chunk| plugin_dirver.tap_generated_chunk(chunk, &self.output_options));

    chunks
      .iter_mut()
      .map(|chunk| {
        let chunk = chunk.render(&self.output_options, &mut self.graph.module_by_id);
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
