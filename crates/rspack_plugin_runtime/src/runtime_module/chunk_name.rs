use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeTemplate, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct ChunkNameRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl ChunkNameRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}chunk_name",
        runtime_template.runtime_module_prefix()
      )),
      None,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ChunkNameRuntimeModule {
  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    if let Some(chunk_ukey) = self.chunk {
      let chunk = compilation.build_chunk_graph_artifact.chunk_by_ukey.expect_get(&chunk_ukey);
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
