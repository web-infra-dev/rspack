use futures::Future;
use indexmap::IndexMap;
use rspack_error::Result;
use rustc_hash::FxHashMap as HashMap;
use tracing::instrument;

use crate::{
  build_chunk_graph::code_splitter::CodeSplitter, incremental::IncrementalPasses, ChunkByUkey,
  ChunkGraph, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, Compilation,
};

#[derive(Debug, Default)]
pub struct CodeSplittingCache {
  chunk_by_ukey: ChunkByUkey,
  chunk_graph: ChunkGraph,
  chunk_group_by_ukey: ChunkGroupByUkey,
  entrypoints: IndexMap<String, ChunkGroupUkey>,
  async_entrypoints: Vec<ChunkGroupUkey>,
  named_chunk_groups: HashMap<String, ChunkGroupUkey>,
  named_chunks: HashMap<String, ChunkUkey>,
  pub(crate) code_splitter: CodeSplitter,
}

#[instrument(skip_all)]
pub(crate) async fn use_code_splitting_cache<'a, T, F>(
  compilation: &'a mut Compilation,
  task: T,
) -> Result<()>
where
  T: Fn(&'a mut Compilation) -> F,
  F: Future<Output = Result<&'a mut Compilation>>,
{
  if !compilation
    .incremental
    .can_read_mutations(IncrementalPasses::MAKE)
  {
    task(compilation).await?;
    return Ok(());
  }

  let has_change = compilation.has_module_import_export_change();
  if has_change
    && !compilation
      .incremental
      .can_read_mutations(IncrementalPasses::BUILD_CHUNK_GRAPH)
  {
    compilation.chunk_by_ukey = Default::default();
    compilation.chunk_graph = Default::default();
    compilation.chunk_group_by_ukey = Default::default();
    compilation.entrypoints = Default::default();
    compilation.async_entrypoints = Default::default();
    compilation.named_chunk_groups = Default::default();
    compilation.named_chunks = Default::default();
  }

  if !has_change {
    return Ok(());
  }

  task(compilation).await?;
  return Ok(());
}
