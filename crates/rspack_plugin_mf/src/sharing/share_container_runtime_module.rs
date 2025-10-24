use rspack_collections::Identifier;
use rspack_core::{ChunkUkey, Compilation, RuntimeModule, RuntimeModuleStage, impl_runtime_module};

#[impl_runtime_module]
#[derive(Debug)]
pub struct ShareContainerRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl ShareContainerRuntimeModule {
  pub fn new() -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/share_container_federation"),
      None,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ShareContainerRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(
      "__webpack_require__.federation = { instance: undefined,bundlerRuntime: undefined };"
        .to_string(),
    )
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}
