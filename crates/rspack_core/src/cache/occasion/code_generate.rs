use rspack_error::Result;

use crate::{cache::storage, BoxModule, CodeGenerationResult, NormalModuleSource};

type Storage = dyn storage::Storage<CodeGenerationResult>;

#[derive(Debug)]
pub struct CodeGenerateOccasion {
  storage: Option<Box<Storage>>,
}

impl CodeGenerateOccasion {
  pub fn new(storage: Option<Box<Storage>>) -> Self {
    Self { storage }
  }

  #[allow(clippy::unwrap_in_result)]
  pub fn use_cache<'a, G>(
    &self,
    module: &'a BoxModule,
    generator: G,
  ) -> Result<CodeGenerationResult>
  where
    G: Fn(&'a BoxModule) -> Result<CodeGenerationResult>,
  {
    let storage = match &self.storage {
      Some(s) => s,
      // no cache return directly
      None => return generator(module),
    };

    let mut need_cache = false;
    let id = module.identifier();
    if let Some(module) = module.as_normal_module() {
      // only cache normal module
      // TODO cache all module type
      if matches!(module.source(), NormalModuleSource::Unbuild) {
        if let Some(data) = storage.get(&id) {
          return Ok(data);
        }
        // unbuild and no cache is unexpected
        panic!("unexpected unbuild module");
      }
      need_cache = true;
    }

    // run generator and save to cache
    let data = generator(module)?;
    if need_cache {
      storage.set(id, data.clone());
    }
    Ok(data)
  }
}
