use std::sync::LazyLock;

use rspack_cacheable::with::AsMap;
use rspack_core::{
  Compilation, IndexChunkIdMap, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeModuleRuntimeRequirements, RuntimeModuleStage, RuntimeTemplate,
  chunk_graph_chunk::ChunkId, impl_runtime_module,
};

use crate::{extract_runtime_globals_from_ejs, extract_runtime_module_variables_from_ejs};

static CHUNK_PREFETCH_TRIGGER_TEMPLATE: &str = include_str!("runtime/chunk_prefetch_trigger.ejs");
static RUNTIME_MODULE_VARIABLES: LazyLock<Vec<&'static str>> =
  LazyLock::new(|| extract_runtime_module_variables_from_ejs(&[CHUNK_PREFETCH_TRIGGER_TEMPLATE]));
static CHUNK_PREFETCH_TRIGGER_RUNTIME_REQUIREMENTS: LazyLock<RuntimeModuleRuntimeRequirements> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(CHUNK_PREFETCH_TRIGGER_TEMPLATE));

#[impl_runtime_module(runtime_module_variables)]
#[derive(Debug)]
pub struct ChunkPrefetchTriggerRuntimeModule {
  #[cacheable(with=AsMap)]
  chunk_map: IndexChunkIdMap<Vec<ChunkId>>,
}

impl ChunkPrefetchTriggerRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, chunk_map: IndexChunkIdMap<Vec<ChunkId>>) -> Self {
    Self::with_default(runtime_template, chunk_map)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ChunkPrefetchTriggerRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    RUNTIME_MODULE_VARIABLES.as_slice()
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      CHUNK_PREFETCH_TRIGGER_TEMPLATE.to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let source = context.runtime_template.render(
      self.id(),
      Some(serde_json::json!({
        "_chunk_map": &self.chunk_map,
      })),
    )?;
    Ok(source)
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Trigger
  }
  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: CHUNK_PREFETCH_TRIGGER_RUNTIME_REQUIREMENTS.dependencies,
      ..Default::default()
    }
  }
}
