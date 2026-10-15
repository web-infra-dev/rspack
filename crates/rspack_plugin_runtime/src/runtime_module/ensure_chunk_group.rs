use std::sync::LazyLock;

use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleStage,
  RuntimeTemplate, impl_runtime_module,
};

use crate::extract_runtime_module_variables_from_ejs;

static TEMPLATE: &str = include_str!("runtime/ensure_chunk_group.ejs");
static RUNTIME_MODULE_VARIABLES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
  let mut variables = extract_runtime_module_variables_from_ejs(&[TEMPLATE]);
  variables.push("__rspack_hmr_s_blocks");
  variables
});

#[impl_runtime_module]
#[derive(Debug)]
pub struct EnsureChunkGroupRuntimeModule {}

impl EnsureChunkGroupRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for EnsureChunkGroupRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    RUNTIME_MODULE_VARIABLES.as_slice()
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(self.id().to_string(), TEMPLATE.to_string())]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let graph = &compilation.build_chunk_graph_artifact;
    let chunk = graph
      .chunk_by_ukey
      .expect_get(&self.chunk().expect("should have chunk"));
    let blocks = graph
      .chunk_graph
      .get_block_chunks(compilation, Some(chunk.runtime()));
    context.runtime_template.render(
      self.id(),
      Some(serde_json::json!({
        "_blocks": rspack_util::json_stringify(&blocks),
        "_hmr": compilation.hot_module_replacement,
        "_state_expression": context.runtime_template.render_hmr_runtime_state_expression("blocks"),
      })),
    )
  }

  fn runtime_requirements(
    &self,
    compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    let mut dependencies = RuntimeGlobals::ENSURE_CHUNK | RuntimeGlobals::ENSURE_CHUNK_HANDLERS;
    if compilation.hot_module_replacement {
      dependencies.insert(RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX);
    }
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies,
      define: RuntimeGlobals::ENSURE_CHUNK_GROUP,
      force_context: RuntimeGlobals::ENSURE_CHUNK | RuntimeGlobals::ENSURE_CHUNK_GROUP,
      ..Default::default()
    }
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}
