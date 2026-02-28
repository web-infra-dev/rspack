use futures::Future;
use rspack_collections::Identifier;
use rspack_error::{Diagnostic, Result};
use rspack_sources::BoxSource;

use crate::{
  ArtifactExt, Chunk, Compilation, MemoryGCStorage, SourceType,
  incremental::{Incremental, IncrementalPasses},
};

#[derive(Debug, Default)]
pub struct ChunkRenderCacheArtifact {
  storage: Option<MemoryGCStorage<BoxSource>>,
}

impl ArtifactExt for ChunkRenderCacheArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::CHUNK_ASSET;

  fn recover(_incremental: &Incremental, new: &mut Self, old: &mut Self) {
    *new = std::mem::take(old);
    new.start_next_generation();
  }
}

impl ChunkRenderCacheArtifact {
  pub fn new(storage: MemoryGCStorage<BoxSource>) -> Self {
    Self {
      storage: Some(storage),
    }
  }
  pub fn start_next_generation(&self) {
    if let Some(storage) = &self.storage {
      storage.start_next_generation();
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
    let Some(storage) = &self.storage else {
      panic!("ChunkRenderCacheArtifact storage is not set");
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
