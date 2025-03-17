use std::future::Future;

use rspack_collections::Identifier;
use rspack_error::Result;

use crate::{old_cache::storage, CodeGenerationJob, CodeGenerationResult};

type Storage = dyn storage::Storage<CodeGenerationResult>;

#[derive(Debug)]
pub struct CodeGenerateOccasion {
  storage: Option<Box<Storage>>,
}

impl CodeGenerateOccasion {
  pub fn new(storage: Option<Box<Storage>>) -> Self {
    Self { storage }
  }

  pub fn begin_idle(&self) {
    if let Some(s) = &self.storage {
      s.begin_idle();
    }
  }

  // #[tracing::instrument(skip_all, fields(module = ?job.module))]
  pub async fn use_cache<G, F>(
    &self,
    job: &CodeGenerationJob,
    provide: G,
  ) -> (Result<CodeGenerationResult>, bool)
  where
    G: FnOnce() -> F,
    F: Future<Output = Result<CodeGenerationResult>>,
  {
    let storage = match &self.storage {
      Some(s) => s,
      None => {
        let res = provide().await;
        return (res, false);
      }
    };
    let cache_key = Identifier::from(format!("{}|{}", job.module, job.hash.encoded()));
    if let Some(value) = storage.get(&cache_key) {
      (Ok(value), true)
    } else {
      match provide().await {
        Ok(res) => {
          storage.set(cache_key, res.clone());
          (Ok(res), false)
        }
        Err(err) => (Err(err), false),
      }
    }
  }
}
