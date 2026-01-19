use futures::Future;
use rspack_collections::Identifier;
use rspack_error::Result;

use crate::{
  CacheOptions, ChunkGraph, Compilation, CompilerOptions, MemoryGCStorage, ModuleIdentifier,
  RuntimeGlobals, RuntimeSpec, get_runtime_key,
};

#[derive(Debug, Default)]
pub struct ProcessRuntimeRequirementsCacheArtifact {
  storage: Option<MemoryGCStorage<RuntimeGlobals>>,
}

impl ProcessRuntimeRequirementsCacheArtifact {
  pub fn new(options: &CompilerOptions) -> Self {
    Self {
      storage: match &options.cache {
        CacheOptions::Memory { max_generations } => Some(MemoryGCStorage::new(*max_generations)),
        _ => None,
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
    module: ModuleIdentifier,
    runtime: &RuntimeSpec,
    compilation: &Compilation,
    generator: G,
  ) -> Result<RuntimeGlobals>
  where
    G: FnOnce() -> F,
    F: Future<Output = Result<RuntimeGlobals>>,
  {
    let Some(storage) = &self.storage else {
      return generator().await;
    };

    let hash = ChunkGraph::get_module_hash(compilation, module, runtime)
      .expect("should have cgm hash in process_runtime_requirements");
    let cache_key = Identifier::from(format!(
      "{}|{}|{}",
      module,
      hash.encoded(),
      get_runtime_key(runtime)
    ));

    if let Some(value) = storage.get(&cache_key) {
      Ok(value)
    } else {
      let res = generator().await?;
      storage.set(cache_key, res);
      Ok(res)
    }
  }
}
