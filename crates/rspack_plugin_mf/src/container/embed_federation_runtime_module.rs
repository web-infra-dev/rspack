//! # EmbedFederationRuntimeModule
//!
//! Runtime module that wraps the startup function to ensure federation runtime dependencies
//! execute before other modules. Generates a "prevStartup wrapper" pattern with defensive
//! checks that intercepts and modifies the startup execution order.

use std::fmt::Write;

use rspack_cacheable::cacheable;
use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, DependencyId, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
  impl_runtime_module, module_raw,
};
use rspack_error::Result;

#[cacheable]
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct EmbedFederationRuntimeModuleOptions {
  pub collected_dependency_ids: Vec<DependencyId>,
  pub async_startup: bool,
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
    let chunk_ukey = self
      .chunk
      .expect("Chunk should be attached to RuntimeModule");

    let collected_deps = &self.options.collected_dependency_ids;

    if collected_deps.is_empty() {
      return Ok("// No federation runtime dependencies to embed.".into());
    }

    let module_graph = compilation.get_module_graph();
    let mut federation_runtime_modules = Vec::new();

    // Find federation runtime dependencies in this chunk
    for dep_id in collected_deps.iter() {
      if let Some(module_dyn) = module_graph.get_module_by_dependency_id(dep_id) {
        let is_in_chunk = compilation
          .chunk_graph
          .is_module_in_chunk(&module_dyn.identifier(), chunk_ukey);
        if is_in_chunk {
          federation_runtime_modules.push(*dep_id);
        }
      }
    }

    if federation_runtime_modules.is_empty() {
      return Ok("// Federation runtime entry modules not found in this chunk.".into());
    }

    // Generate module execution code for each federation runtime dependency
    let mut runtime_requirements = RuntimeGlobals::default();
    let mut module_executions = String::with_capacity(federation_runtime_modules.len() * 64);

    for dep_id in federation_runtime_modules {
      let module_str = module_raw(compilation, &mut runtime_requirements, &dep_id, "", false);
      module_executions.push_str("		");
      module_executions.push_str(&module_str);
      module_executions.push('\n');
    }

    // Build startup wrappers. Always wrap STARTUP; also wrap STARTUP_ENTRYPOINT when async startup
    // is enabled so runtimeChunk: "single" flows still initialize federation runtime.
    let startup = RuntimeGlobals::STARTUP.name();
    let startup_entry = RuntimeGlobals::STARTUP_ENTRYPOINT.name();
    let wrap_async = self.options.async_startup;

    let mut code = String::with_capacity(256 + module_executions.len());

    writeln!(code, "var __webpack_require__mf_has_run = false;").expect("write to string");
    writeln!(
      code,
      "var __webpack_require__mf_startup_once = function() {"
    )
    .unwrap();
    writeln!(code, "	if (__webpack_require__mf_has_run) return;").unwrap();
    writeln!(code, "	__webpack_require__mf_has_run = true;").unwrap();
    code.push_str(&module_executions);
    writeln!(code, "};").unwrap();

    writeln!(code, "function __webpack_require__mf_wrapStartup(prev) {").unwrap();
    writeln!(
      code,
      "	var fn = typeof prev === 'function' ? prev : undefined;"
    )
    .unwrap();
    writeln!(code, "	return function() {").unwrap();
    writeln!(code, "		__webpack_require__mf_startup_once();").unwrap();
    writeln!(code, "		if (typeof fn === 'function') {").unwrap();
    writeln!(code, "			return fn.apply(this, arguments);").unwrap();
    writeln!(code, "		}").unwrap();
    writeln!(code, "	};").unwrap();
    writeln!(code, "}").unwrap();

    writeln!(
      code,
      "{startup} = __webpack_require__mf_wrapStartup({startup});"
    )
    .unwrap();
    if wrap_async {
      writeln!(
        code,
        "{startup_entry} = __webpack_require__mf_wrapStartup({startup_entry});"
      )
      .unwrap();
    }

    Ok(code)
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Trigger // Run after RemoteRuntimeModule and StartupChunkDependenciesRuntimeModule
  }
}
