use futures::future::BoxFuture;
use rspack_collections::Identifier;
use rspack_error::Result;
use rspack_hash::RspackHashDigest;

use crate::{old_cache::storage, CodeGenerationResult};
use crate::{ModuleIdentifier, RuntimeSpec};

type Storage = dyn storage::Storage<CodeGenerationResult>;

#[derive(Debug)]
pub struct CodeGenerateOccasion {
  storage: Option<Box<Storage>>,
}

impl CodeGenerateOccasion {
  pub fn new(storage: Option<Box<Storage>>) -> Self {
    Self { storage }
  }

  // #[tracing::instrument(skip_all, fields(module = ?job.module))]
  pub async fn use_cache(
    &self,
    module: ModuleIdentifier,
    hash: RspackHashDigest,
    runtimes: Vec<RuntimeSpec>,
    provide: BoxFuture<'_, Result<CodeGenerationResult>>,
  ) -> (Result<CodeGenerationResult>, Vec<RuntimeSpec>, bool) {
    let storage = match &self.storage {
      Some(s) => s,
      None => {
        let res = provide.await;
        return (res, runtimes, false);
      }
    };
    let cache_key = Identifier::from(format!("{}|{}", module, hash.encoded()));
    if let Some(value) = storage.get(&cache_key) {
      (Ok(value), runtimes, true)
    } else {
      match provide.await {
        Ok(res) => {
          storage.set(cache_key, res.clone());
          (Ok(res), runtimes, false)
        }
        Err(err) => (Err(err), runtimes, false),
      }
    }
  }
}
