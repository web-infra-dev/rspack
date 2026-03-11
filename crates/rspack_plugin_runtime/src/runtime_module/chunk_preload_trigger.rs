use std::sync::LazyLock;

use rspack_cacheable::with::AsMap;
use rspack_core::{
  Compilation, IndexChunkIdMap, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeModuleStage, RuntimeTemplate, chunk_graph_chunk::ChunkId, impl_runtime_module,
};

use crate::extract_runtime_globals_from_ejs;

static CHUNK_PRELOAD_TRIGGER_TEMPLATE: &str = include_str!("runtime/chunk_preload_trigger.ejs");
static CHUNK_PRELOAD_TRIGGER_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(CHUNK_PRELOAD_TRIGGER_TEMPLATE));

#[impl_runtime_module]
#[derive(Debug)]
pub struct ChunkPreloadTriggerRuntimeModule {
  #[cacheable(with=AsMap)]
  chunk_map: IndexChunkIdMap<Vec<ChunkId>>,
}

impl ChunkPreloadTriggerRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, chunk_map: IndexChunkIdMap<Vec<ChunkId>>) -> Self {
    Self::with_default(runtime_template, chunk_map)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ChunkPreloadTriggerRuntimeModule {
  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      CHUNK_PRELOAD_TRIGGER_TEMPLATE.to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let source = context.runtime_template.render(
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
    *CHUNK_PRELOAD_TRIGGER_RUNTIME_REQUIREMENTS
  }
}
