use rspack_collections::Identifier;
use rspack_error::Result;

use crate::{
  get_runtime_key, CodeGenerationJob, Module, ModuleIdentifier, RuntimeSpec, RuntimeSpecSet,
};
use crate::{old_cache::storage, BoxModule, CodeGenerationResult, Compilation, NormalModuleSource};

type Storage = dyn storage::Storage<CodeGenerationResult>;

#[derive(Debug)]
pub struct CodeGenerateOccasion {
  storage: Option<Box<Storage>>,
}

impl CodeGenerateOccasion {
  pub fn new(storage: Option<Box<Storage>>) -> Self {
    Self { storage }
  }

  pub fn use_cache<'a>(
    &self,
    job: CodeGenerationJob,
    provide: impl Fn(ModuleIdentifier, &RuntimeSpec) -> Result<CodeGenerationResult>,
  ) -> Result<(CodeGenerationResult, Vec<RuntimeSpec>)> {
    let storage = match &self.storage {
      Some(s) => s,
      None => {
        let res = provide(job.module, &job.runtime)?;
        return Ok((res, job.runtimes));
      }
    };
    let cache_key = Identifier::from(format!("{}|{}", job.module, job.hash.encoded()));
    if let Some(value) = storage.get(&cache_key) {
      return Ok((value, job.runtimes));
    } else {
      let res = provide(job.module, &job.runtime)?;
      storage.set(cache_key, res.clone());
      return Ok((res, job.runtimes));
    }
  }
}
