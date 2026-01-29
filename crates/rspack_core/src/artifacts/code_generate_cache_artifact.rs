use futures::Future;
use rspack_collections::Identifier;
use rspack_error::Result;

use crate::{
  ArtifactExt, CacheOptions, CodeGenerationJob, CodeGenerationResult, CompilerOptions,
  MemoryGCStorage,
  incremental::{Incremental, IncrementalPasses},
};

#[derive(Debug, Default)]
pub struct CodeGenerateCacheArtifact {
  storage: Option<MemoryGCStorage<CodeGenerationResult>>,
}

impl ArtifactExt for CodeGenerateCacheArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::MODULES_CODEGEN;

  fn recover(_incremental: &Incremental, new: &mut Self, old: &mut Self) {
    *new = std::mem::take(old);
    new.start_next_generation();
  }
}

impl CodeGenerateCacheArtifact {
  pub fn new(options: &CompilerOptions) -> Self {
    Self {
      storage: match &options.cache {
        CacheOptions::Memory { max_generations } => Some(MemoryGCStorage::new(*max_generations)),
        CacheOptions::Persistent(_) => Some(MemoryGCStorage::new(1)),
        CacheOptions::Disabled => None,
      },
    }
  }

  pub fn start_next_generation(&self) {
    if let Some(storage) = &self.storage {
      storage.start_next_generation();
    }
  }

  pub async fn use_cache<G, F>(
    &self,
    job: &CodeGenerationJob,
    generator: G,
  ) -> (Result<CodeGenerationResult>, bool)
  where
    G: FnOnce() -> F,
    F: Future<Output = Result<CodeGenerationResult>>,
  {
    let Some(storage) = &self.storage else {
      let res = generator().await;
      return (res, false);
    };

    let cache_key = Identifier::from(format!("{}|{}", job.module, job.hash.encoded()));
    if let Some(value) = storage.get(cache_key) {
      (Ok(value), true)
    } else {
      match generator().await {
        Ok(res) => {
          storage.set(cache_key, res.clone());
          (Ok(res), false)
        }
        Err(err) => (Err(err), false),
      }
    }
  }
}
