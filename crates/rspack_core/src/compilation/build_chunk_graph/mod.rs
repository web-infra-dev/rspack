// use rspack_core::Bundle;
// use rspack_core::ChunkGraph;

use tracing::instrument;

use self::code_splitter::CodeSplitter;
use crate::Compilation;
pub(crate) mod code_splitter;
pub(crate) mod incremental;
pub(crate) mod pass;

#[instrument("Compilation:build_chunk_graph", skip_all)]
pub fn build_chunk_graph(compilation: &mut Compilation) -> rspack_error::Result<()> {
  let module_graph = compilation.get_module_graph();
  let all_modules = module_graph.modules_keys().copied().collect::<Vec<_>>();
  let entrypoint_count = compilation.entries.len();
  let estimated_chunk_group_count = entrypoint_count + module_graph.blocks().len();
  // Each initial entry can also create a separate runtime chunk.
  let estimated_chunk_count = estimated_chunk_group_count + entrypoint_count;

  // Each async block can create a chunk/group. This is an upper-bound estimate:
  // inlined/unreachable blocks and shared names can reduce the actual counts.
  let artifact = &mut compilation.build_chunk_graph_artifact;
  let chunk_graph_modules = &mut artifact.chunk_graph.chunk_graph_module_by_module_identifier;
  chunk_graph_modules.reserve(all_modules.len().saturating_sub(chunk_graph_modules.len()));
  let chunk_graph_chunks = &mut artifact.chunk_graph.chunk_graph_chunk_by_chunk_ukey;
  chunk_graph_chunks.reserve(estimated_chunk_count.saturating_sub(chunk_graph_chunks.len()));
  artifact
    .chunk_by_ukey
    .reserve(estimated_chunk_count.saturating_sub(artifact.chunk_by_ukey.len()));
  artifact
    .chunk_group_by_ukey
    .reserve(estimated_chunk_group_count.saturating_sub(artifact.chunk_group_by_ukey.len()));
  artifact
    .entrypoints
    .reserve(entrypoint_count.saturating_sub(artifact.entrypoints.len()));

  // TODO: heuristic incremental update is temporarily disabled
  // Original code:
  // let enable_incremental = compilation
  //   .incremental
  //   .mutations_readable(IncrementalPasses::BUILD_CHUNK_GRAPH);
  let enable_incremental = false;
  let mut splitter = if enable_incremental {
    std::mem::take(&mut compilation.build_chunk_graph_artifact.code_splitter)
  } else {
    let mut splitter = CodeSplitter::default();
    splitter
      .chunk_group_info_map
      .reserve(estimated_chunk_group_count);
    splitter
      .chunk_group_infos
      .reserve(estimated_chunk_group_count);
    splitter.ordinal_by_module.reserve(all_modules.len());
    splitter.mask_by_chunk.reserve(estimated_chunk_count);
    splitter
  };

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
