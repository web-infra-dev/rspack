use futures::Future;
use rspack_collections::Identifier;
use rspack_error::Result;

use crate::{
  CacheOptions, CodeGenerationJob, CodeGenerationResult, CompilerOptions, MemoryGCStorage,
};

#[derive(Debug, Default)]
pub struct CodeGenerateCacheArtifact {
  storage: Option<MemoryGCStorage<CodeGenerationResult>>,
}

impl CodeGenerateCacheArtifact {
  pub fn new(options: &CompilerOptions) -> Self {
    Self {
      storage: match &options.cache {
        CacheOptions::Memory { max_generations } => {
          Some(MemoryGCStorage::new(max_generations.unwrap_or(1)))
        }
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
    if let Some(value) = storage.get(&cache_key) {
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
