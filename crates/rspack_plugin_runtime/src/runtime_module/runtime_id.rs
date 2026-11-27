use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct RuntimeIdRuntimeModule {}

impl RuntimeIdRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for RuntimeIdRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      define: { RuntimeGlobals::RUNTIME_ID },
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      include_str!("runtime/runtime_id.ejs").to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let graph = &context.compilation.build_chunk_graph_artifact;
    let chunk_ukey = self.chunk().expect("should attach chunk for runtime_id");
    let runtime = graph.chunk_by_ukey.expect_get(&chunk_ukey).runtime();
    if runtime.len() > 1 {
      panic!("RuntimeIdRuntimeModule must be in a single runtime");
    }
    let id = graph
      .chunk_graph
      .get_runtime_id(runtime.iter().next().expect("At least one runtime"));
    context
      .runtime_template
      .render(self.id(), Some(serde_json::json!({ "_runtime_id": id })))
  }
}
