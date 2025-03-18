// use rspack_core::Bundle;
// use rspack_core::ChunkGraph;

use tracing::instrument;

use crate::{incremental::IncrementalPasses, Compilation};

pub(crate) mod code_splitter;
pub(crate) mod incremental;
pub(crate) mod new_code_splitter;
mod remove_available_modules;

#[instrument("Compilation:build_chunk_graph", skip_all)]
pub fn build_chunk_graph(compilation: &mut Compilation) -> rspack_error::Result<()> {
  let enable_incremental = compilation
    .incremental
    .can_read_mutations(IncrementalPasses::BUILD_CHUNK_GRAPH);
  let mut splitter = if enable_incremental {
    std::mem::take(&mut compilation.code_splitting_cache.code_splitter)
  } else {
    Default::default()
  };

  splitter.update_with_compilation(compilation)?;

  if !enable_incremental || splitter.chunk_group_infos.is_empty() {
    let inputs = splitter.prepare_input_entrypoints_and_modules(compilation)?;
    splitter.prepare_entries(inputs, compilation)?;
  }

  splitter.split(compilation)?;
  splitter.remove_orphan(compilation)?;

  // make sure all module (weak dependency particularly) has a cgm
  let ids = compilation
    .get_module_graph()
    .modules()
    .keys()
    .copied()
    .collect::<Vec<_>>();

  for module_identifier in ids {
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
