// use rspack_core::Bundle;
// use rspack_core::ChunkGraph;

use tracing::instrument;

use crate::{Compilation, incremental::IncrementalPasses};
pub(crate) mod code_splitter;
pub(crate) mod incremental;
pub(crate) mod pass;

// TODO: heuristic incremental updates are temporarily disabled. Keep cache
// preparation tied to the same switch, including on the initial compilation.
pub(crate) const ENABLE_HEURISTIC_INCREMENTAL: bool = false;

#[instrument("Compilation:build_chunk_graph", skip_all)]
pub fn build_chunk_graph(compilation: &mut Compilation) -> rspack_error::Result<()> {
  let enable_incremental = ENABLE_HEURISTIC_INCREMENTAL
    && compilation
      .incremental
      .mutations_readable(IncrementalPasses::BUILD_CHUNK_GRAPH);
  let mut splitter = if enable_incremental {
    std::mem::take(&mut compilation.build_chunk_graph_artifact.code_splitter)
  } else {
    Default::default()
  };

  let all_modules = compilation
    .get_module_graph()
    .modules_keys()
    .copied()
    .collect::<Vec<_>>();
  compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .reserve_modules(all_modules.len());

  // Make sure all modules (particularly weak dependencies) have a CGM before splitting.
  for module_identifier in &all_modules {
    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .add_module(*module_identifier);
  }

  splitter.prepare(&all_modules, compilation)?;

  splitter.update_with_compilation(compilation)?;

  if !enable_incremental || splitter.chunk_group_infos.is_empty() {
    let inputs = splitter.prepare_input_entrypoints_and_modules(&all_modules, compilation)?;
    splitter.prepare_entries(inputs, compilation)?;
  }

  splitter.split(compilation)?;

  // remove empty chunk groups
  splitter.remove_orphan(compilation)?;

  // Orphan cleanup may remove the CGM of a module that remains in the module graph.
  for module_identifier in all_modules {
    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .add_module(module_identifier);
  }

  compilation
    .build_chunk_graph_artifact
    .set_code_splitter(splitter);

  Ok(())
}
