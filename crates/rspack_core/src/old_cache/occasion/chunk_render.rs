use futures::Future;
use rspack_collections::Identifier;
use rspack_error::{Diagnostic, Result};
use rspack_sources::BoxSource;

use crate::{old_cache::storage, Chunk, Compilation, SourceType};

type Storage = dyn storage::Storage<BoxSource>;

#[derive(Debug)]
pub struct ChunkRenderOccasion {
  storage: Option<Box<Storage>>,
}

impl ChunkRenderOccasion {
  pub fn new(storage: Option<Box<Storage>>) -> Self {
    Self { storage }
  }

  pub fn begin_idle(&self) {
    if let Some(s) = &self.storage {
      s.begin_idle();
    }
  }

  pub async fn use_cache<G, F>(
    &self,
    compilation: &Compilation,
    chunk: &Chunk,
    source_type: &SourceType,
    generator: G,
  ) -> Result<(BoxSource, Vec<Diagnostic>)>
  where
    G: FnOnce() -> F,
    F: Future<Output = Result<(BoxSource, Vec<Diagnostic>)>>,
  {
    let storage = match &self.storage {
      Some(s) => s,
      // no cache return directly
      None => return generator().await,
    };

    let Some(content_hash) =
      chunk.content_hash_by_source_type(&compilation.chunk_hashes_artifact, source_type)
    else {
      return generator().await;
    };
    let cache_key = Identifier::from(content_hash.encoded());
    if let Some(value) = storage.get(&cache_key) {
      Ok((value, Vec::new()))
    } else {
      let res = generator().await?;
      storage.set(cache_key, res.0.clone());
      Ok(res)
    }
  }
}
