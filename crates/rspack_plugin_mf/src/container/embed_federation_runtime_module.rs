//! # EmbedFederationRuntimeModule
//!
//! Runtime module that wraps the startup function to ensure federation runtime dependencies
//! execute before other modules. Generates a "prevStartup wrapper" pattern with defensive
//! checks that intercepts and modifies the startup execution order.

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
    let mut module_executions = String::with_capacity(federation_runtime_modules.len() * 50);

    for dep_id in federation_runtime_modules {
      let module_str = module_raw(compilation, &mut runtime_requirements, &dep_id, "", false);
      module_executions.push_str("\t\t");
      module_executions.push_str(&module_str);
      module_executions.push('\n');
    }
    // Remove trailing newline
    if !module_executions.is_empty() {
      module_executions.pop();
    }

    let compat_helper = r#"if (typeof __webpack_require__.n !== "function") {
	__webpack_require__.n = function (module) {
		var getter = module && module.__esModule ? function () { return module["default"]; } : function () { return module; };
		getter.a = getter;
		return getter;
	};
}
"#;

    // Generate prevStartup wrapper pattern with defensive checks
    let startup = RuntimeGlobals::STARTUP.name();

    // When async startup is enabled, only expose installRuntime and let the entry startup code call it
    // When async startup is disabled, call the runtime directly in startup (backwards compat with main)
    let startup_call = if self.options.async_startup {
      // Async startup enabled: don't call runtime in startup, expose installRuntime instead
      ""
    } else {
      // Async startup disabled: call runtime directly in startup (backwards compat with main)
      "\trunFederationRuntime();\n"
    };

    let result = format!(
      r#"var prevStartup = {startup};
var hasRun = false;
if (typeof __webpack_require__.g !== "undefined") {{
	if (typeof __webpack_require__.g.importScripts === "undefined") {{
		__webpack_require__.g.importScripts = function() {{ return false; }};
	}}
	if (typeof __webpack_require__.g.location === "undefined") {{
		__webpack_require__.g.location = {{
			href: "http://localhost/_rspack_placeholder_.js",
			toString: function() {{ return this.href; }}
		}};
	}}
	if (typeof __webpack_require__.g.document === "undefined") {{
		__webpack_require__.g.document = {{
			currentScript: {{ tagName: "script", src: "http://localhost/_rspack_placeholder_.js" }},
			getElementsByTagName: function() {{ return [{{ src: "http://localhost/_rspack_placeholder_.js" }}]; }}
		}};
	}}
}}
function runFederationRuntime() {{
	if (!hasRun) {{
		hasRun = true;
{compat_helper}
{module_executions}
	}}
}}
__webpack_require__.federation = __webpack_require__.federation || {{}};
__webpack_require__.federation.installRuntime = runFederationRuntime;
{startup} = function() {{
{startup_call}	if (typeof prevStartup === 'function') {{
		return prevStartup.apply(this, arguments);
	}}
	if (typeof prevStartup !== 'undefined') {{
		return prevStartup;
	}}
	console.warn('[MF] Invalid prevStartup');
}};
"#
    );

    Ok(result)
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::from(11) // Run after RemoteRuntimeModule (stage 10)
  }
}
