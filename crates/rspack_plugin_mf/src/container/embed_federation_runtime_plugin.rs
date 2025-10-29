//! # EmbedFederationRuntimePlugin
//!
//! Plugin that manages federation runtime initialization by adding STARTUP requirements
//! to federation-enabled chunks and injecting EmbedFederationRuntimeModule into runtime chunks.
//! Also handles explicit startup calls for entry chunks that delegate to runtime chunks.

use std::sync::{Arc, Mutex};

use rspack_core::{
  ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements, CompilationParams,
  CompilationRuntimeRequirementInTree, CompilerCompilation, DependencyId, ModuleIdentifier, Plugin,
  RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{JavascriptModulesRenderStartup, JsPlugin, RenderSource};
use rspack_sources::{ConcatSource, RawStringSource, SourceExt};
use rustc_hash::FxHashSet;

use super::{
  container_entry_module::ContainerEntryModule,
  embed_federation_runtime_module::{
    EmbedFederationRuntimeModule, EmbedFederationRuntimeModuleOptions,
  },
  federation_modules_plugin::{AddFederationRuntimeDependencyHook, FederationModulesPlugin},
  federation_runtime_dependency::FederationRuntimeDependency,
};

struct FederationRuntimeDependencyCollector {
  collected_dependency_ids: Arc<Mutex<FxHashSet<DependencyId>>>,
}

#[async_trait::async_trait]
impl AddFederationRuntimeDependencyHook for FederationRuntimeDependencyCollector {
  async fn run(&self, dependency: &FederationRuntimeDependency) -> Result<()> {
    self
      .collected_dependency_ids
      .lock()
      .expect("Failed to lock collected_dependency_ids")
      .insert(dependency.id);
    Ok(())
  }
}

#[plugin]
#[derive(Debug)]
pub struct EmbedFederationRuntimePlugin {
  collected_dependency_ids: Arc<Mutex<FxHashSet<DependencyId>>>,
}

impl EmbedFederationRuntimePlugin {
  pub fn new() -> Self {
    Self::new_inner(Arc::new(Mutex::new(FxHashSet::default())))
  }

  /// Check if the chunk is a container entry chunk (should NOT use async startup)
  fn is_container_entry_chunk(compilation: &Compilation, chunk_ukey: &ChunkUkey) -> bool {
    let entries = compilation
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey);

    // Inspect the last entry module (final entry) and its chunk group metadata
    if let Some((module_id, chunk_group_ukey)) = entries.iter().next_back() {
      let module_graph = compilation.get_module_graph();
      if let Some(module) = module_graph.module_by_identifier(module_id) {
        return module.as_any().is::<ContainerEntryModule>();
      }
      let chunk_group = compilation.chunk_group_by_ukey.expect_get(chunk_group_ukey);
      if let Some(entry_options) = chunk_group.kind.get_entry_options()
        && entry_options.library.is_some()
      {
        return true;
      }
    }
    false
  }
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for EmbedFederationRuntimePlugin)]
async fn additional_chunk_runtime_requirements_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  // Skip build time chunks
  if chunk.name() == Some("build time chunk") {
    return Ok(());
  }

  // Check if chunk needs federation runtime support
  let has_runtime = chunk.has_runtime(&compilation.chunk_group_by_ukey);
  let has_entry_modules = compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey)
    > 0;

  // Federation is enabled for runtime chunks or entry chunks
  let is_enabled = has_runtime || has_entry_modules;
  let is_container_entry_chunk = Self::is_container_entry_chunk(compilation, chunk_ukey);
  let use_async_startup =
    compilation.options.experiments.mf_async_startup && !is_container_entry_chunk;

  if is_enabled {
    // Add STARTUP or STARTUP_ENTRYPOINT based on mf_async_startup experiment
    if use_async_startup {
      runtime_requirements.insert(RuntimeGlobals::STARTUP_ENTRYPOINT);
      runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    } else {
      runtime_requirements.insert(RuntimeGlobals::STARTUP);
    }
  }

  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for EmbedFederationRuntimePlugin)]
async fn runtime_requirement_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  _runtime_requirements: &RuntimeGlobals,
  _runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  // Skip build time chunks
  if chunk.name() == Some("build time chunk") {
    return Ok(None);
  }

  // Only inject EmbedFederationRuntimeModule into runtime chunks
  let has_runtime = chunk.has_runtime(&compilation.chunk_group_by_ukey);
  if has_runtime {
    // Collect federation dependencies snapshot
    let collected_ids_snapshot = self
      .collected_dependency_ids
      .lock()
      .expect("Failed to lock collected_dependency_ids")
      .iter()
      .cloned()
      .collect::<Vec<DependencyId>>();

    let emro = EmbedFederationRuntimeModuleOptions {
      collected_dependency_ids: collected_ids_snapshot,
    };

    // Inject EmbedFederationRuntimeModule
    compilation.add_runtime_module(
      chunk_ukey,
      Box::new(EmbedFederationRuntimeModule::new(emro)),
    )?;
  }

  Ok(None)
}

#[plugin_hook(CompilerCompilation for EmbedFederationRuntimePlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let collector = FederationRuntimeDependencyCollector {
    collected_dependency_ids: Arc::clone(&self.collected_dependency_ids),
  };

  let federation_hooks = FederationModulesPlugin::get_compilation_hooks(compilation);

  federation_hooks
    .add_federation_runtime_dependency
    .lock()
    .await
    .tap(collector);

  // Register render startup hook, patches entrypoints
  let js_hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  js_hooks
    .write()
    .await
    .render_startup
    .tap(render_startup::new(self));

  Ok(())
}

#[plugin_hook(JavascriptModulesRenderStartup for EmbedFederationRuntimePlugin)]
async fn render_startup(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _module: &ModuleIdentifier,
  render_source: &mut RenderSource,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  // Skip build time chunks
  if chunk.name() == Some("build time chunk") {
    return Ok(());
  }

  // Skip container entry chunks (they should NOT use async startup)
  if Self::is_container_entry_chunk(compilation, chunk_ukey) {
    return Ok(());
  }

  // Only process chunks that have federation dependencies
  let collected_deps = self
    .collected_dependency_ids
    .lock()
    .expect("Failed to lock collected_dependency_ids")
    .iter()
    .cloned()
    .collect::<Vec<DependencyId>>();
  let has_federation_deps = !collected_deps.is_empty();

  if !has_federation_deps {
    return Ok(());
  }

  let has_runtime = chunk.has_runtime(&compilation.chunk_group_by_ukey);
  let has_entry_modules = compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey)
    > 0;

  // Runtime chunks with entry modules: JavaScript plugin handles startup naturally
  if has_runtime && has_entry_modules {
    return Ok(());
  }

  // Entry chunks delegating to runtime need explicit federation initialization
  if !has_runtime && has_entry_modules {
    let is_esm_output = compilation.options.output.module;
    let mut startup_with_call = ConcatSource::default();

    if compilation.options.experiments.mf_async_startup {
      let chunk_id = chunk.expect_id(&compilation.chunk_ids_artifact);
      let chunk_id_str = serde_json::to_string(chunk_id).expect("invalid chunk_id");

      if is_esm_output {
        // ESM async mode: wrap execution in Promise with async/await syntax
        // Extract any import statements from the source to keep them at top level
        let source_str = render_source.source.source().into_string_lossy();
        let mut imports = Vec::new();
        let mut execution = Vec::new();
        for line in source_str.lines() {
          if line.trim().starts_with("import ") {
            imports.push(line);
          } else {
            execution.push(line);
          }
        }

        // Add imports at top level
        for import in imports {
          startup_with_call.add(RawStringSource::from(format!("{}\n", import)));
        }

        // Add federation initialization and Promise wrapper
        startup_with_call.add(RawStringSource::from(format!(
          "const {}Promise = Promise.resolve().then(async () => {{\n",
          RuntimeGlobals::EXPORTS.name()
        )));
        startup_with_call.add(RawStringSource::from(
          "  // Initialize federation runtime\n",
        ));
        startup_with_call.add(RawStringSource::from(
          "  if (typeof __webpack_require__.x === 'function') {\n",
        ));
        startup_with_call.add(RawStringSource::from(
          "    await __webpack_require__.x();\n",
        ));
        startup_with_call.add(RawStringSource::from("  }\n"));
        startup_with_call.add(RawStringSource::from("  const promises = [];\n"));
        startup_with_call.add(RawStringSource::from("  const handlers = [\n"));
        startup_with_call.add(RawStringSource::from("    function(chunkId, promises) {\n"));
        startup_with_call.add(RawStringSource::from("      return (__webpack_require__.f.consumes || function(chunkId, promises) {})(chunkId, promises);\n"));
        startup_with_call.add(RawStringSource::from("    },\n"));
        startup_with_call.add(RawStringSource::from("    function(chunkId, promises) {\n"));
        startup_with_call.add(RawStringSource::from("      return (__webpack_require__.f.remotes || function(chunkId, promises) {})(chunkId, promises);\n"));
        startup_with_call.add(RawStringSource::from("    }\n"));
        startup_with_call.add(RawStringSource::from("  ];\n"));
        startup_with_call.add(RawStringSource::from(format!(
          "  await Promise.all(handlers.reduce(function(p, handler) {{ return handler({}, p), p; }}, promises));\n",
          chunk_id_str
        )));
        // Add the execution code (non-import lines) indented
        for line in execution {
          startup_with_call.add(RawStringSource::from(format!("  {}\n", line)));
        }
        startup_with_call.add(RawStringSource::from(format!(
          "  return {};\n",
          RuntimeGlobals::EXPORTS.name()
        )));
        startup_with_call.add(RawStringSource::from("});\n"));
        startup_with_call.add(RawStringSource::from(format!(
          "export default await {}Promise;\n",
          RuntimeGlobals::EXPORTS.name()
        )));
        render_source.source = startup_with_call.boxed();
      } else {
        // CJS output: use function-based Promise pattern
        startup_with_call.add(RawStringSource::from(
          "\n// Initialize federation runtime\n",
        ));
        startup_with_call.add(RawStringSource::from(
          "var runtimeInitialization = undefined;\n",
        ));
        startup_with_call.add(RawStringSource::from(
          "if (typeof __webpack_require__.x === 'function') {\n",
        ));
        startup_with_call.add(RawStringSource::from(
          "  runtimeInitialization = __webpack_require__.x();\n",
        ));
        startup_with_call.add(RawStringSource::from("}\n"));
        startup_with_call.add(RawStringSource::from("var promises = [];\n"));
        startup_with_call.add(RawStringSource::from(format!(
          "var {} = Promise.resolve(runtimeInitialization).then(function() {{\n",
          RuntimeGlobals::EXPORTS.name()
        )));
        startup_with_call.add(RawStringSource::from("  var handlers = [\n"));
        startup_with_call.add(RawStringSource::from("    function(chunkId, promises) {\n"));
        startup_with_call.add(RawStringSource::from("      return (__webpack_require__.f.consumes || function(chunkId, promises) {})(chunkId, promises);\n"));
        startup_with_call.add(RawStringSource::from("    },\n"));
        startup_with_call.add(RawStringSource::from("    function(chunkId, promises) {\n"));
        startup_with_call.add(RawStringSource::from("      return (__webpack_require__.f.remotes || function(chunkId, promises) {})(chunkId, promises);\n"));
        startup_with_call.add(RawStringSource::from("    }\n"));
        startup_with_call.add(RawStringSource::from("  ];\n"));
        startup_with_call.add(RawStringSource::from(format!(
          "  return Promise.all(handlers.reduce(function(p, handler) {{ return handler({}, p), p; }}, promises));\n",
          chunk_id_str
        )));
        startup_with_call.add(RawStringSource::from("}).then(function() {\n"));
        // Add the original source but change it to return the value instead of declaring __webpack_exports__
        startup_with_call.add(RawStringSource::from("  return (function() {\n"));
        startup_with_call.add(render_source.source.clone());
        startup_with_call.add(RawStringSource::from("    return __webpack_exports__;\n"));
        startup_with_call.add(RawStringSource::from("  })();\n"));
        startup_with_call.add(RawStringSource::from("});\n"));
        render_source.source = startup_with_call.boxed();
      }
    } else {
      // Standard sync startup call - prepend to original
      startup_with_call.add(RawStringSource::from_static(
        "\n// Federation startup call\n",
      ));
      startup_with_call.add(RawStringSource::from(format!(
        "{}();\n",
        RuntimeGlobals::STARTUP.name()
      )));
      startup_with_call.add(render_source.source.clone());
      render_source.source = startup_with_call.boxed();
    }
  }

  Ok(())
}

impl Plugin for EmbedFederationRuntimePlugin {
  fn name(&self) -> &'static str {
    "EmbedFederationRuntimePlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements_tree::new(self));
    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirement_in_tree::new(self));
    Ok(())
  }
}

impl Default for EmbedFederationRuntimePlugin {
  fn default() -> Self {
    Self::new()
  }
}
