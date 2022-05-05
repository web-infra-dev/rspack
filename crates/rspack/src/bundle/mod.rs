use std::{collections::HashMap, sync::Arc};

use crate::{
  bundler::BundleOptions, module_graph, plugin_driver::PluginDriver, structs::OutputChunk,
  utils::get_compiler,
};
use rspack_shared::ModuleGraph;
use tracing::instrument;

use self::split_chunks::split_chunks;
pub mod split_chunks;

#[derive(Debug)]
pub struct Bundle {
  pub output_options: Arc<BundleOptions>,
}

impl Bundle {
  pub fn new(output_options: Arc<BundleOptions>) -> Self {
    Self { output_options }
  }

  #[instrument(skip(self, plugin_dirver, graph))]
  pub fn generate(
    &mut self,
    plugin_dirver: &PluginDriver,
    graph: &mut ModuleGraph,
  ) -> HashMap<String, OutputChunk> {
    let mut chunks = split_chunks(&graph);

    chunks.iter_mut().for_each(|chunk| {
      chunk.id = chunk.generate_id(&self.output_options);
    });

    chunks
      .iter()
      .for_each(|chunk| plugin_dirver.tap_generated_chunk(chunk, &self.output_options));
    let compiler = get_compiler();

    chunks
      .iter_mut()
      .map(|chunk| {
        let chunk = chunk.render(
          &self.output_options,
          &mut graph.module_by_id,
          compiler.clone(),
        );
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
