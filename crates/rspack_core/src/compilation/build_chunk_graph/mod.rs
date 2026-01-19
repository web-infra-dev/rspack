// use rspack_core::Bundle;
// use rspack_core::ChunkGraph;

use tracing::instrument;

use crate::Compilation;
pub(crate) mod code_splitter;
pub(crate) mod incremental;
pub(crate) mod pass;

#[instrument("Compilation:build_chunk_graph", skip_all)]
pub fn build_chunk_graph(compilation: &mut Compilation) -> rspack_error::Result<()> {
  let mut splitter = code_splitter::CodeSplitter::default();

  let all_modules = compilation
    .get_module_graph()
    .modules()
    .keys()
    .copied()
    .collect::<Vec<_>>();

  splitter.prepare(&all_modules, compilation)?;

  let inputs = splitter.prepare_input_entrypoints_and_modules(&all_modules, compilation)?;
  splitter.prepare_entries(inputs, compilation)?;

  splitter.split(compilation)?;

  // remove empty chunk groups
  splitter.remove_orphan(compilation)?;

  // make sure all module (weak dependency particularly) has a cgm
  for module_identifier in all_modules {
    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .add_module(module_identifier)
  }

  // save splitter for no_change detection in next rebuild
  compilation.build_chunk_graph_artifact.code_splitter = splitter;

  Ok(())
}
