use futures::Future;
use indexmap::IndexMap;
use rspack_collections::Database;
use rspack_error::Result;
use rustc_hash::FxHashMap as HashMap;
use tracing::instrument;

use crate::{
  build_chunk_graph::code_splitter::CodeSplitter, incremental::IncrementalPasses, Chunk,
  ChunkGraph, ChunkGroup, ChunkGroupUkey, ChunkUkey, Compilation,
};

#[derive(Debug, Default)]
pub struct CodeSplittingCache {
  chunk_by_ukey: Database<Chunk>,
  chunk_graph: ChunkGraph,
  chunk_group_by_ukey: Database<ChunkGroup>,
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
  if !has_change
    || compilation
      .incremental
      .can_read_mutations(IncrementalPasses::BUILD_CHUNK_GRAPH)
  {
    let cache = &mut compilation.code_splitting_cache;
    rayon::scope(|s| {
      s.spawn(|_| compilation.chunk_by_ukey = cache.chunk_by_ukey.clone());
      s.spawn(|_| compilation.chunk_graph = cache.chunk_graph.clone());
      s.spawn(|_| compilation.chunk_group_by_ukey = cache.chunk_group_by_ukey.clone());
      s.spawn(|_| compilation.entrypoints = cache.entrypoints.clone());
      s.spawn(|_| compilation.async_entrypoints = cache.async_entrypoints.clone());
      s.spawn(|_| compilation.named_chunk_groups = cache.named_chunk_groups.clone());
      s.spawn(|_| compilation.named_chunks = cache.named_chunks.clone());
    });

    if !has_change {
      return Ok(());
    }
  }

  let compilation = task(compilation).await?;
  let cache = &mut compilation.code_splitting_cache;
  rayon::scope(|s| {
    s.spawn(|_| cache.chunk_by_ukey = compilation.chunk_by_ukey.clone());
    s.spawn(|_| cache.chunk_graph = compilation.chunk_graph.clone());
    s.spawn(|_| cache.chunk_group_by_ukey = compilation.chunk_group_by_ukey.clone());
    s.spawn(|_| cache.entrypoints = compilation.entrypoints.clone());
    s.spawn(|_| cache.async_entrypoints = compilation.async_entrypoints.clone());
    s.spawn(|_| cache.named_chunk_groups = compilation.named_chunk_groups.clone());
    s.spawn(|_| cache.named_chunks = compilation.named_chunks.clone());
  });
  Ok(())
}
