use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, DependencyId, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
  impl_runtime_module, module_raw,
};
use rspack_error::Result;

#[impl_runtime_module]
#[derive(Debug)]
pub struct EntryFederationRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  dependency: DependencyId,
}

impl EntryFederationRuntimeModule {
  pub fn new(dependency: DependencyId) -> Self {
    Self::with_default(
      Identifier::from("module_federation/entry_runtime"),
      None,
      dependency,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for EntryFederationRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Trigger
  }

  async fn generate(&self, compilation: &Compilation) -> Result<String> {
    // ensure the module id is resolved
    let mut runtime_requirements = RuntimeGlobals::default();
    let entry_factory = module_raw(
      compilation,
      &mut runtime_requirements,
      &self.dependency,
      "",
      false,
    );
    Ok(format!(
      r#"var __module_federation_entry_runtime__ = {entry_factory};
if (typeof __module_federation_entry_runtime__ === "function") {{
  __module_federation_entry_runtime__();
}}
"#
    ))
  }
}
