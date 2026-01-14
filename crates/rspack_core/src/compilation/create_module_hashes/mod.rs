use rspack_collections::IdentifierSet;
use rspack_error::{Result, ToStringResultToRspackResultExt};
use tracing::instrument;

use crate::{ChunkGraph, Compilation, RuntimeSpecMap, incremental::Mutation};

#[instrument("Compilation:create_module_hashes", skip_all)]
pub async fn create_module_hashes(
  compilation: &mut Compilation,
  modules: IdentifierSet,
) -> Result<()> {
  let mg = compilation.get_module_graph();
  let chunk_graph = &compilation.chunk_graph;
  let chunk_by_ukey = &compilation.chunk_by_ukey;

  let results = rspack_futures::scope::<_, Result<_>>(|token| {
    for module_identifier in modules {
      let s = unsafe { token.used((&*compilation, &mg, chunk_graph, chunk_by_ukey)) };
      s.spawn(
        move |(compilation, mg, chunk_graph, chunk_by_ukey)| async move {
          let mut hashes = RuntimeSpecMap::new();
          let module = mg
            .module_by_identifier(&module_identifier)
            .expect("should have module");
          for runtime in chunk_graph.get_module_runtimes_iter(module_identifier, chunk_by_ukey) {
            let hash = module.get_runtime_hash(compilation, Some(runtime)).await?;
            hashes.set(runtime.clone(), hash);
          }
          Ok((module_identifier, hashes))
        },
      );
    }
  })
  .await
  .into_iter()
  .map(|r| r.to_rspack_result())
  .collect::<Result<Vec<_>>>()?;

  for result in results {
    let (module, hashes) = result?;
    if ChunkGraph::set_module_hashes(compilation, module, hashes)
      && let Some(mut mutations) = compilation.incremental.mutations_write()
    {
      mutations.add(Mutation::ModuleSetHashes { module });
    }
  }
  Ok(())
}
