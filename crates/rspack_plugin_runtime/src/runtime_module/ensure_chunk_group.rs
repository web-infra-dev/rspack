use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
};

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
    &[]
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      include_str!("runtime/ensure_chunk_group.ejs").to_string(),
    )]
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
    let groups = graph
      .chunk_graph
      .get_async_chunk_groups(compilation, Some(chunk.runtime()));
    context.runtime_template.render(
      self.id(),
      Some(serde_json::json!({ "_groups": rspack_util::json_stringify(&groups) })),
    )
  }

  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::ENSURE_CHUNK
        | RuntimeGlobals::ENSURE_CHUNK_HANDLERS
        | RuntimeGlobals::REQUIRE_SCOPE,
      define: RuntimeGlobals::ENSURE_CHUNK_GROUP,
      force_context: RuntimeGlobals::ENSURE_CHUNK | RuntimeGlobals::ENSURE_CHUNK_GROUP,
      ..Default::default()
    }
  }
}
