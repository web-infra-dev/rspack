use futures::Future;
use indexmap::IndexMap;
use rspack_error::Result;
use rustc_hash::FxHashMap as HashMap;
use tracing::instrument;

use crate::{
  build_chunk_graph::code_splitter::CodeSplitter, incremental::IncrementalPasses, ChunkByUkey,
  ChunkGraph, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, Compilation, ModuleIdentifier,
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
  pub(crate) module_idx: HashMap<ModuleIdentifier, (u32, u32)>,
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
    || (std::env::var("LEGACY_CODE_SPLITTING").is_ok()
      && compilation
        .incremental
        .can_read_mutations(IncrementalPasses::BUILD_CHUNK_GRAPH))
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

    let module_idx = cache.module_idx.clone();
    let mut module_graph = compilation.get_module_graph_mut();
    for (m, (pre, post)) in module_idx {
      let Some(mgm) = module_graph.module_graph_module_by_identifier_mut(&m) else {
        continue;
      };

      mgm.pre_order_index = Some(pre);
      mgm.post_order_index = Some(post);
    }

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

  let mg = compilation.get_module_graph();
  let mut map = HashMap::default();
  for m in mg.modules().keys() {
    let Some(mgm) = mg.module_graph_module_by_identifier(m) else {
      continue;
    };

    let (Some(pre), Some(post)) = (mgm.pre_order_index, mgm.post_order_index) else {
      continue;
    };

    map.insert(*m, (pre, post));
  }
  let cache = &mut compilation.code_splitting_cache;
  cache.module_idx = map;
  Ok(())
}
