use futures::Future;
use rspack_error::Result;
use rspack_identifier::Identifier;

use crate::{old_cache::storage, Chunk, Compilation, NormalModuleSource, RenderManifestEntry};

type Storage = dyn storage::Storage<Vec<RenderManifestEntry>>;

#[derive(Debug)]
pub struct CreateChunkAssetsOccasion {
  storage: Option<Box<Storage>>,
}

impl CreateChunkAssetsOccasion {
  pub fn new(storage: Option<Box<Storage>>) -> Self {
    Self { storage }
  }

  pub async fn use_cache<'a, G, F>(
    &self,
    compilation: &Compilation,
    chunk: &Chunk,
    generator: G,
  ) -> Result<Vec<RenderManifestEntry>>
  where
    G: Fn() -> F,
    F: Future<Output = Result<Vec<RenderManifestEntry>>>,
  {
    let storage = match &self.storage {
      Some(s) => s,
      // no cache return directly
      None => return generator().await,
    };

    let chunk_id = Identifier::from(chunk.expect_id());
    let modules = &compilation
      .chunk_graph
      .get_chunk_graph_chunk(&chunk.ukey)
      .modules;
    let is_cache_valid = modules.iter().all(|module_id| {
      matches!(
        compilation
          .get_module_graph()
          .module_by_identifier(module_id)
          .and_then(|m| m.as_normal_module())
          .map(|m| matches!(m.source(), NormalModuleSource::Unbuild)),
        Some(true)
      )
    });

    if is_cache_valid {
      // read
      if let Some(data) = storage.get(&chunk_id) {
        return Ok(data);
      }
    }
    // run generator and save to cache
    let data = generator().await?;
    // TODO sometime may not run save
    storage.set(chunk_id, data.clone());
    Ok(data)
  }
}
