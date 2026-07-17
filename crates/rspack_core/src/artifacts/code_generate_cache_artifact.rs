use futures::Future;
use rspack_collections::Identifier;
use rspack_error::Result;

use crate::{
  ArtifactExt, CacheOptions, CodeGenerationJob, CodeGenerationResult, CompilerOptions,
  MemoryGCStorage, RuntimeSpec,
  incremental::{Incremental, IncrementalPasses},
  runtime_mode::RuntimeMode,
};

fn code_generation_cache_key(job: &CodeGenerationJob, runtime_mode: RuntimeMode) -> Identifier {
  // Code generation may choose whether to share runtime helpers based on the number of runtime
  // trees that reuse this module-hash result, so that group size is part of the cache identity.
  let runtime_group_size = RuntimeSpec::from_runtimes(job.runtimes.iter()).len();
  Identifier::from(format!(
    "{}|{}|{}|{runtime_group_size}",
    job.module,
    job.hash.encoded(),
    runtime_mode
  ))
}

#[derive(Debug, Default)]
pub struct CodeGenerateCacheArtifact {
  storage: Option<MemoryGCStorage<CodeGenerationResult>>,
  runtime_mode: RuntimeMode,
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
      runtime_mode: options.experiments.runtime_mode,
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

    let cache_key = code_generation_cache_key(job, self.runtime_mode);
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

#[cfg(test)]
mod tests {
  use rspack_hash::RspackHashDigest;

  use super::*;
  use crate::ModuleIdentifier;

  fn job_with_runtimes(runtimes: Vec<RuntimeSpec>) -> CodeGenerationJob {
    CodeGenerationJob {
      module: ModuleIdentifier::from("module"),
      hash: RspackHashDigest::from("hash"),
      runtime: runtimes[0].clone(),
      runtimes,
      scope: None,
    }
  }

  #[test]
  fn cache_key_accounts_for_runtime_group_size() {
    let one_runtime = job_with_runtimes(vec![RuntimeSpec::from_iter(["a".into()])]);
    let four_runtimes = job_with_runtimes(vec![
      RuntimeSpec::from_iter(["a".into()]),
      RuntimeSpec::from_iter(["b".into(), "c".into(), "d".into()]),
    ]);
    let four_other_runtimes = job_with_runtimes(vec![RuntimeSpec::from_iter([
      "w".into(),
      "x".into(),
      "y".into(),
      "z".into(),
    ])]);

    let one_key = code_generation_cache_key(&one_runtime, RuntimeMode::Webpack);
    let four_key = code_generation_cache_key(&four_runtimes, RuntimeMode::Webpack);
    let four_other_key = code_generation_cache_key(&four_other_runtimes, RuntimeMode::Webpack);

    assert_ne!(one_key, four_key);
    assert_eq!(four_key, four_other_key);
  }
}
