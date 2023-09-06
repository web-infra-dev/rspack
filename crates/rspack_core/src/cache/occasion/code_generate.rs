use rspack_error::Result;
use rspack_identifier::Identifier;

use crate::{cache::storage, BoxModule, CodeGenerationResult, Compilation, NormalModuleSource};

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
    compilation: &Compilation,
    generator: G,
  ) -> Result<(CodeGenerationResult, bool)>
  where
    G: Fn(&'a BoxModule) -> Result<CodeGenerationResult>,
  {
    let storage = match &self.storage {
      Some(s) => s,
      // no cache return directly
      None => return Ok((generator(module)?, false)),
    };

    let mut cache_id = None;

    if let Some(normal_module) = module.as_normal_module() {
      // only cache normal module
      // TODO: cache all module type
      let id = Identifier::from(compilation.chunk_graph.get_module_graph_hash(
        module,
        &compilation.module_graph,
        true,
      ));

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
    let data = generator(module)?;
    if let Some(id) = cache_id {
      storage.set(id, data.clone());
    }
    Ok((data, false))
  }
}
