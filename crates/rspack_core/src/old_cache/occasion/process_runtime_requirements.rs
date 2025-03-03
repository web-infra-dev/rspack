use rspack_collections::Identifier;
use rspack_error::Result;

use crate::old_cache::storage;
use crate::{
  get_runtime_key, ChunkGraph, Compilation, ModuleIdentifier, RuntimeGlobals, RuntimeSpec,
};

type Storage = dyn storage::Storage<RuntimeGlobals>;

#[derive(Debug)]
pub struct ProcessRuntimeRequirementsOccasion {
  storage: Option<Box<Storage>>,
}

impl ProcessRuntimeRequirementsOccasion {
  pub fn new(storage: Option<Box<Storage>>) -> Self {
    Self { storage }
  }

  pub fn begin_idle(&self) {
    if let Some(s) = &self.storage {
      s.begin_idle();
    }
  }

  // #[tracing::instrument(skip_all, fields(module = ?module))]
  pub fn use_cache(
    &self,
    module: ModuleIdentifier,
    runtime: &RuntimeSpec,
    compilation: &Compilation,
    provide: impl Fn(ModuleIdentifier, &RuntimeSpec) -> Result<RuntimeGlobals>,
  ) -> Result<RuntimeGlobals> {
    let storage = match &self.storage {
      Some(s) => s,
      None => {
        let res = provide(module, runtime)?;
        return Ok(res);
      }
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
      let res = provide(module, runtime)?;
      storage.set(cache_key, res);
      Ok(res)
    }
  }
}
