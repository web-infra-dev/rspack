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
    // Check if the federation runtime module is in this chunk
    let chunk_ukey = self
      .chunk
      .expect("Chunk should be attached to RuntimeModule");

    let module_graph = compilation.get_module_graph();
    let is_in_chunk =
      if let Some(module_dyn) = module_graph.get_module_by_dependency_id(&self.dependency) {
        compilation
          .chunk_graph
          .is_module_in_chunk(&module_dyn.identifier(), chunk_ukey)
      } else {
        false
      };

    if !is_in_chunk {
      return Ok("// Federation runtime entry module not found in this chunk.".into());
    }

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
