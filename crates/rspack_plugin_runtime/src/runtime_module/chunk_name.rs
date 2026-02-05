use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeTemplate, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct ChunkNameRuntimeModule {}

impl ChunkNameRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ChunkNameRuntimeModule {
  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    if let Some(chunk_ukey) = self.chunk {
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
      Ok(format!(
        "{} = {};",
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::CHUNK_NAME),
        serde_json::to_string(&chunk.name()).expect("Invalid json string")
      ))
    } else {
      unreachable!("should attach chunk for css_loading")
    }
  }
}
