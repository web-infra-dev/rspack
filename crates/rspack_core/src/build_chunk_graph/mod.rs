// use rspack_core::Bundle;
// use rspack_core::ChunkGraph;

use tracing::instrument;

use crate::Compilation;

pub(crate) mod code_splitter;
pub(crate) mod incremental;

#[instrument(skip_all)]
pub(crate) fn build_chunk_graph(compilation: &mut Compilation) -> rspack_error::Result<()> {
  // let mut splitter = code_splitter::CodeSplitter::new(compilation);
  let mut splitter = compilation.code_splitting_cache.code_splitter.clone();
  splitter.update_with_compilation(compilation)?;

  if !compilation.options.incremental().code_splitting_enabled()
    || splitter.chunk_group_infos.is_empty()
  {
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
