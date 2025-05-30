use rspack_cacheable::cacheable;
use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module, module_raw, ChunkUkey, Compilation, DependencyId, RuntimeGlobals,
  RuntimeModule, RuntimeModuleStage,
};
use rspack_error::Result;

#[cacheable]
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
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
    println!("🔧 EmbedFederationRuntimeModule::generate called");

    let chunk_ukey = self
      .chunk
      .expect("Chunk should be attached to RuntimeModule");

    let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
    println!("   - chunk: {:?}", chunk.name());

    let collected_deps = &self.options.collected_dependency_ids;
    println!("   - collected_deps count: {}", collected_deps.len());

    if collected_deps.is_empty() {
      println!("   ❌ No federation runtime dependencies to embed");
      return Ok("// No federation runtime dependencies to embed.".to_string());
    }

    let module_graph = compilation.get_module_graph();
    let mut federation_runtime_modules = Vec::new();

    // Find ALL federation runtime dependencies in this chunk
    for dep_id in collected_deps.iter() {
      println!("   - checking dep_id: {:?}", dep_id);
      if let Some(module_dyn) = module_graph.get_module_by_dependency_id(dep_id) {
        let is_in_chunk = compilation
          .chunk_graph
          .is_module_in_chunk(&module_dyn.identifier(), chunk_ukey);
        println!("     - module found, is_in_chunk: {}", is_in_chunk);
        if is_in_chunk {
          federation_runtime_modules.push(*dep_id);
        }
      } else {
        println!("     - module not found in module graph");
      }
    }

    println!(
      "   - federation_runtime_modules count: {}",
      federation_runtime_modules.len()
    );

    if federation_runtime_modules.is_empty() {
      println!("   ❌ Federation runtime entry modules not found in this chunk");
      return Ok("// Federation runtime entry modules not found in this chunk.".to_string());
    }

    // Generate the module raw code for each federation runtime dependency
    let mut runtime_requirements = RuntimeGlobals::default();
    let mut module_executions = Vec::new();

    for dep_id in federation_runtime_modules {
      let module_str = module_raw(compilation, &mut runtime_requirements, &dep_id, "", false);
      module_executions.push(format!("\t\t{}", module_str));
    }

    // Generate the oldStartup wrapper pattern with all federation runtime modules
    let result = format!(
      r#"var oldStartup = {startup};
var hasRun = false;
{startup} = function() {{
	if (!hasRun) {{
		hasRun = true;
{module_executions}
	}}
	return oldStartup();
}};"#,
      startup = RuntimeGlobals::STARTUP.name(),
      module_executions = module_executions.join("\n")
    );

    println!(
      "   ✅ Generated oldStartup wrapper with {} federation modules: {} chars",
      module_executions.len(),
      result.len()
    );
    Ok(result)
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::from(11) // Attach + 1, ensures it runs after RemoteRuntimeModule (which uses Attach=10)
  }
}
