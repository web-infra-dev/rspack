use rspack_error::Result;
use rspack_identifier::Identifier;

use crate::{get_runtime_key, RuntimeSpec, RuntimeSpecSet};
use crate::{old_cache::storage, BoxModule, CodeGenerationResult, Compilation, NormalModuleSource};

type Storage = dyn storage::Storage<Vec<(CodeGenerationResult, RuntimeSpec)>>;

#[derive(Debug)]
pub struct CodeGenerateOccasion {
  storage: Option<Box<Storage>>,
}

impl CodeGenerateOccasion {
  pub fn new(storage: Option<Box<Storage>>) -> Self {
    Self { storage }
  }

  pub fn use_cache<'a, G>(
    &self,
    module: &'a BoxModule,
    runtimes: RuntimeSpecSet,
    compilation: &Compilation,
    generator: G,
  ) -> Result<(Vec<(CodeGenerationResult, RuntimeSpec)>, bool)>
  where
    G: Fn(&'a BoxModule, RuntimeSpecSet) -> Result<Vec<(CodeGenerationResult, RuntimeSpec)>>,
  {
    let storage = match &self.storage {
      Some(s) => s,
      // no cache return directly
      None => return Ok((generator(module, runtimes)?, false)),
    };

    let mut cache_id = None;

    if let Some(normal_module) = module.as_normal_module() {
      // only cache normal module
      // TODO: cache all module type
      let mut id = String::default();
      for runtime in runtimes.values() {
        id.push_str(&compilation.chunk_graph.get_module_graph_hash(
          module,
          compilation,
          Some(runtime),
          true,
        ));
        id.push_str(&get_runtime_key(runtime.clone()));
      }
      let id = Identifier::from(id);

      // currently no need to separate module hash by runtime
      if let Some(data) = storage.get(&id) {
        return Ok((data, true));
      }

      if matches!(normal_module.source(), NormalModuleSource::Unbuild) {
        // unbuild and no cache is unexpected
        panic!("unexpected unbuild module");
      }
      cache_id = Some(id);
    }

    // run generator and save to cache
    let data = generator(module, runtimes)?;
    if let Some(id) = cache_id {
      storage.set(id, data.clone());
    }
    Ok((data, false))
  }
}
