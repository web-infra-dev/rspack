use std::collections::HashSet;

use rkyv::{Archive, Deserialize, Serialize};
use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module, module_raw, ChunkGraph, ChunkUkey, Compilation, DependencyId,
  ModuleIdentifier, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
};
use rspack_error::Result;

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct EmbedFederationRuntimeModuleOptions {
  pub collected_dependency_ids: Vec<DependencyId>,
}

#[impl_runtime_module]
#[derive(Debug)]
pub struct EmbedFederationRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  options: EmbedFederationRuntimeModuleOptions,
}

impl EmbedFederationRuntimeModule {
  pub fn new(options: EmbedFederationRuntimeModuleOptions) -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/embed_federation_runtime"),
      None,
      options,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for EmbedFederationRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, compilation: &Compilation) -> Result<String> {
    let _chunk_ukey = self
      .chunk
      .expect("Chunk should be attached to RuntimeModule");

    let collected_deps = &self.options.collected_dependency_ids;
    if collected_deps.is_empty() {
      return Ok("// No federation runtime dependencies to embed.".to_string());
    }

    let mut found_module_identifier: Option<ModuleIdentifier> = None;
    let mut target_dep_id: Option<DependencyId> = None;
    let module_graph = compilation.get_module_graph();

    for dep_id in collected_deps.iter() {
      if let Some(module_dyn) = module_graph.get_module_by_dependency_id(dep_id) {
        let current_chunk_ukey = self.chunk.unwrap();
        if compilation
          .chunk_graph
          .is_module_in_chunk(&module_dyn.identifier(), current_chunk_ukey)
        {
          found_module_identifier = Some(module_dyn.identifier());
          target_dep_id = Some(*dep_id);
          break;
        }
      }
    }

    if found_module_identifier.is_none() {
      return Ok("// Federation runtime entry module not found in this chunk.".to_string());
    }
    let _target_module_identifier = found_module_identifier.unwrap();
    let final_dep_id =
      target_dep_id.expect("Dependency ID should be found if module identifier is found");

    let mut runtime_requirements = RuntimeGlobals::default();

    let module_str = module_raw(
      compilation,
      &mut runtime_requirements,
      &final_dep_id,
      &"".to_string(),
      false,
    );

    let result = format!(
      "var oldStartup = {startup_var};
var hasRun = false;
{startup_var} = function() {{
  if (!hasRun) {{
    hasRun = true;
    {module_str}
  }}
  if(oldStartup) return oldStartup.apply(this, arguments);
}};
",
      startup_var = RuntimeGlobals::STARTUP,
      module_str = module_str
    );
    Ok(result)
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}
