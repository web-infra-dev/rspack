use rspack_core::{
  RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct BaseUriRuntimeModule {}

impl BaseUriRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for BaseUriRuntimeModule {
  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let base_uri = self
      .chunk
      .and_then(|ukey| {
        compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .get(&ukey)
      })
      .and_then(|chunk| {
        chunk.get_entry_options(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
      })
      .and_then(|options| options.base_uri.as_ref())
      .and_then(|base_uri| serde_json::to_string(base_uri).ok())
      .unwrap_or_else(|| "undefined".to_string());
    Ok(format!(
      "{} = {};\n",
      context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::BASE_URI),
      base_uri
    ))
  }
}
