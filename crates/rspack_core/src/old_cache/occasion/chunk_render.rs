use futures::Future;
use rspack_collections::Identifier;
use rspack_error::Result;

use crate::{old_cache::storage, ChunkRenderResult, ChunkUkey, Compilation};

type Storage = dyn storage::Storage<ChunkRenderResult>;

#[derive(Debug)]
pub struct ChunkRenderOccasion {
  storage: Option<Box<Storage>>,
}

impl ChunkRenderOccasion {
  pub fn new(storage: Option<Box<Storage>>) -> Self {
    Self { storage }
  }

  pub async fn use_cache<G, F>(
    &self,
    compilation: &Compilation,
    chunk: &ChunkUkey,
    generator: G,
  ) -> Result<ChunkRenderResult>
  where
    G: Fn() -> F,
    F: Future<Output = Result<ChunkRenderResult>>,
  {
    let storage = match &self.storage {
      Some(s) => s,
      // no cache return directly
      None => return generator().await,
    };

    let chunk = compilation.chunk_by_ukey.expect_get(chunk);
    let chunk_hash = chunk
      .hash(&compilation.chunk_hashes_results)
      .expect("should have chunk hash");
    let cache_key = Identifier::from(chunk_hash.encoded());
    if let Some(value) = storage.get(&cache_key) {
      Ok(value)
    } else {
      let res = generator().await?;
      storage.set(cache_key, res.clone());
      Ok(res)
    }
  }
}
