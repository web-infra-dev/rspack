// use rspack_core::Bundle;
// use rspack_core::ChunkGraph;

use tracing::instrument;

use crate::{Compilation, incremental::IncrementalPasses};
pub(crate) mod artifact;
mod available_modules;
pub(crate) mod code_splitter;
pub(crate) mod incremental;
pub(crate) mod new_code_splitter;

#[instrument("Compilation:build_chunk_graph", skip_all)]
pub fn build_chunk_graph(compilation: &mut Compilation) -> rspack_error::Result<()> {
  let enable_incremental = compilation
    .incremental
    .mutations_readable(IncrementalPasses::BUILD_CHUNK_GRAPH);
  let mut splitter = if enable_incremental {
    std::mem::take(&mut compilation.code_splitting_cache.code_splitter)
  } else {
    Default::default()
  };

  let all_modules = compilation
    .get_module_graph()
    .modules()
    .keys()
    .copied()
    .collect::<Vec<_>>();

  splitter.prepare(&all_modules, compilation)?;

  splitter.update_with_compilation(compilation)?;

  if !enable_incremental || splitter.chunk_group_infos.is_empty() {
    let inputs = splitter.prepare_input_entrypoints_and_modules(&all_modules, compilation)?;
    splitter.prepare_entries(inputs, compilation)?;
  }

  splitter.split(compilation)?;

  // remove empty chunk groups
  splitter.remove_orphan(compilation)?;

  // make sure all module (weak dependency particularly) has a cgm
  for module_identifier in all_modules {
    compilation.chunk_graph.add_module(module_identifier)
  }

  compilation.code_splitting_cache.code_splitter = splitter;

  Ok(())
}

#[instrument(skip_all)]
pub fn build_chunk_graph_new(compilation: &mut Compilation) -> rspack_error::Result<()> {
  new_code_splitter::code_split(compilation)?;
  Ok(())
}
