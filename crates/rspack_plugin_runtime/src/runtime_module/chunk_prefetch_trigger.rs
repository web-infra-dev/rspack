use std::{hash::BuildHasherDefault, sync::LazyLock};

use indexmap::IndexMap;
use rspack_cacheable::with::AsMap;
use rspack_collections::Identifier;
use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage, RuntimeTemplate,
  chunk_graph_chunk::ChunkId, impl_runtime_module,
};
use rustc_hash::FxHasher;

use crate::extract_runtime_globals_from_ejs;

const CHUNK_PREFETCH_TRIGGER_TEMPLATE: &str = include_str!("runtime/chunk_prefetch_trigger.ejs");
const CHUNK_PREFETCH_TRIGGER_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(CHUNK_PREFETCH_TRIGGER_TEMPLATE));

#[impl_runtime_module]
#[derive(Debug)]
pub struct ChunkPrefetchTriggerRuntimeModule {
  id: Identifier,
  #[cacheable(with=AsMap)]
  chunk_map: IndexMap<ChunkId, Vec<ChunkId>, BuildHasherDefault<FxHasher>>,
}

impl ChunkPrefetchTriggerRuntimeModule {
  pub fn new(
    runtime_template: &RuntimeTemplate,
    chunk_map: IndexMap<ChunkId, Vec<ChunkId>, BuildHasherDefault<FxHasher>>,
  ) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}chunk_prefetch_trigger",
        runtime_template.runtime_module_prefix()
      )),
      chunk_map,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ChunkPrefetchTriggerRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      CHUNK_PREFETCH_TRIGGER_TEMPLATE.to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let source = compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "_chunk_map": &self.chunk_map,
      })),
    )?;
    Ok(source)
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Trigger
  }

  fn additional_runtime_requirements(&self, _compilation: &Compilation) -> RuntimeGlobals {
    *CHUNK_PREFETCH_TRIGGER_RUNTIME_REQUIREMENTS
  }
}
