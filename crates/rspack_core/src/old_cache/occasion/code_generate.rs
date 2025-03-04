use rspack_collections::Identifier;
use rspack_error::Result;

use crate::{old_cache::storage, CodeGenerationResult};
use crate::{CodeGenerationJob, ModuleIdentifier, RuntimeSpec};

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
  pub fn use_cache(
    &self,
    job: CodeGenerationJob,
    provide: impl Fn(ModuleIdentifier, &RuntimeSpec) -> Result<CodeGenerationResult>,
  ) -> (Result<CodeGenerationResult>, Vec<RuntimeSpec>, bool) {
    let storage = match &self.storage {
      Some(s) => s,
      None => {
        let res = provide(job.module, &job.runtime);
        return (res, job.runtimes, false);
      }
    };
    let cache_key = Identifier::from(format!("{}|{}", job.module, job.hash.encoded()));
    if let Some(value) = storage.get(&cache_key) {
      (Ok(value), job.runtimes, true)
    } else {
      match provide(job.module, &job.runtime) {
        Ok(res) => {
          storage.set(cache_key, res.clone());
          (Ok(res), job.runtimes, false)
        }
        Err(err) => (Err(err), job.runtimes, false),
      }
    }
  }
}
