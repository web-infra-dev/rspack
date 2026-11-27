use std::sync::LazyLock;

use itertools::Itertools;
use rspack_core::{
  ChunkUkey, Compilation, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeModuleRuntimeRequirements, RuntimeModuleStage, RuntimeTemplate, impl_runtime_module,
};

use crate::extract_runtime_globals_from_ejs;

static CHUNK_PREFETCH_STARTUP_TEMPLATE: &str = include_str!("runtime/chunk_prefetch_startup.ejs");
static CHUNK_PREFETCH_STARTUP_RUNTIME_REQUIREMENTS: LazyLock<RuntimeModuleRuntimeRequirements> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(CHUNK_PREFETCH_STARTUP_TEMPLATE));

#[impl_runtime_module]
#[derive(Debug)]
pub struct ChunkPrefetchStartupRuntimeModule {
  startup_chunks: Vec<(Vec<ChunkUkey>, Vec<ChunkUkey>)>,
}

impl ChunkPrefetchStartupRuntimeModule {
  pub fn new(
    runtime_template: &RuntimeTemplate,
    startup_chunks: Vec<(Vec<ChunkUkey>, Vec<ChunkUkey>)>,
  ) -> Self {
    Self::with_default(runtime_template, startup_chunks)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ChunkPrefetchStartupRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      CHUNK_PREFETCH_STARTUP_TEMPLATE.to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let chunks = &context.compilation.build_chunk_graph_artifact.chunk_by_ukey;
    let chunk_ukey = self.chunk().expect("chunk do not attached");

    let startup_chunks = self
      .startup_chunks
      .iter()
      .map(|(group_chunks, child_chunks)| {
        let group_chunk_ids = group_chunks
          .iter()
          .filter(|&&ukey| ukey == chunk_ukey)
          .filter_map(|ukey| chunks.expect_get(ukey).id())
          .collect_vec();

        let child_chunk_ids = child_chunks
          .iter()
          .filter_map(|ukey| chunks.expect_get(ukey).id())
          .collect_vec();

        serde_json::json!({
          "chunk_ids": group_chunk_ids,
          "child_chunk_ids": child_chunk_ids,
        })
      })
      .collect_vec();

    context.runtime_template.render(
      self.id(),
      Some(serde_json::json!({ "_startup_chunks": startup_chunks })),
    )
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Trigger
  }
  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: CHUNK_PREFETCH_STARTUP_RUNTIME_REQUIREMENTS.dependencies,
      ..Default::default()
    }
  }
}
