use futures::Future;
use rspack_database::Database;
use rspack_error::Result;
use rustc_hash::FxHashMap as HashMap;
use tracing::instrument;

use crate::{fast_drop, Chunk, ChunkGraph, ChunkGroup, ChunkGroupUkey, ChunkUkey, Compilation};

#[derive(Debug, Default)]
pub struct CodeSplittingCache {
  chunk_by_ukey: Database<Chunk>,
  chunk_graph: ChunkGraph,
  chunk_group_by_ukey: Database<ChunkGroup>,
  entrypoints: HashMap<String, ChunkGroupUkey>,
  named_chunk_groups: HashMap<String, ChunkGroupUkey>,
  named_chunks: HashMap<String, ChunkUkey>,
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
  let is_incremental_rebuild = compilation.options.is_incremental_rebuild();
  if !is_incremental_rebuild {
    task(compilation).await?;
    return Ok(());
  }

  if !compilation.has_module_import_export_change {
    fast_drop((
      std::mem::replace(
        &mut compilation.chunk_by_ukey,
        compilation.code_splitting_cache.chunk_by_ukey.clone(),
      ),
      std::mem::replace(
        &mut compilation.chunk_graph,
        compilation.code_splitting_cache.chunk_graph.clone(),
      ),
      std::mem::replace(
        &mut compilation.chunk_group_by_ukey,
        compilation.code_splitting_cache.chunk_group_by_ukey.clone(),
      ),
      std::mem::replace(
        &mut compilation.entrypoints,
        compilation.code_splitting_cache.entrypoints.clone(),
      ),
      std::mem::replace(
        &mut compilation.named_chunk_groups,
        compilation.code_splitting_cache.named_chunk_groups.clone(),
      ),
      std::mem::replace(
        &mut compilation.named_chunks,
        compilation.code_splitting_cache.named_chunks.clone(),
      ),
    ));
    return Ok(());
  }

  let compilation = task(compilation).await?;
  fast_drop((
    std::mem::replace(
      &mut compilation.code_splitting_cache.chunk_by_ukey,
      compilation.chunk_by_ukey.clone(),
    ),
    std::mem::replace(
      &mut compilation.code_splitting_cache.chunk_graph,
      compilation.chunk_graph.clone(),
    ),
    std::mem::replace(
      &mut compilation.code_splitting_cache.chunk_group_by_ukey,
      compilation.chunk_group_by_ukey.clone(),
    ),
    std::mem::replace(
      &mut compilation.code_splitting_cache.entrypoints,
      compilation.entrypoints.clone(),
    ),
    std::mem::replace(
      &mut compilation.code_splitting_cache.named_chunk_groups,
      compilation.named_chunk_groups.clone(),
    ),
    std::mem::replace(
      &mut compilation.code_splitting_cache.named_chunks,
      compilation.named_chunks.clone(),
    ),
  ));
  Ok(())
}
