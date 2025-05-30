//! # EmbedFederationRuntimePlugin
//!
//! This plugin manages the initialization and embedding of Module Federation runtime dependencies
//! across different types of chunks in the compilation. It ensures that federation runtime is
//! properly initialized before any federated modules are executed.
//!
//! ## What Gets Added to Different Chunk Types:
//!
//! ### Runtime Chunks:
//! - **EmbedFederationRuntimeModule**: Injected into all runtime chunks that have federation dependencies
//! - **STARTUP Runtime Requirement**: Signals that a startup function should be created for entry module execution
//! - **Purpose**: The runtime module wraps the startup function to ensure federation modules execute first
//!
//! ### Entry Chunks (Delegating to Runtime):
//! - **Explicit Startup Calls**: Injected via `render_startup` hook to call the startup function in delegated runtime
//! - **STARTUP Runtime Requirement**: Signals need for startup function creation in the runtime chunk
//! - **Purpose**: Ensures the delegated runtime chunk's startup function gets called to initialize federation
//!
//! ### Entry Chunks (With Own Runtime):
//! - **STARTUP Runtime Requirement**: Signals that a startup function should be created for entry module execution
//! - **No Interference**: JavaScript plugin creates startup function naturally, EmbedFederationRuntimeModule wraps it
//! - **Purpose**: Leverages existing webpack startup mechanism while adding federation initialization
//!
//! ## STARTUP Runtime Requirement Explained:
//! The STARTUP runtime requirement tells the JavaScript modules plugin to:
//! 1. Create a `__webpack_require__.startup` function that executes entry modules
//! 2. Instead of inlining entry execution, wrap it in a callable function
//! 3. Allow runtime modules (like ours) to intercept and modify the startup process
//! 4. Enable deferred or conditional execution of entry modules
//!
//! ## Activation Conditions:
//! - Plugin only activates when federation runtime dependencies are present
//! - Skips "build time chunk" to avoid development-only chunks
//! - Only processes chunks that are federation-enabled (runtime OR entry modules present)
//!
//! ## Hook Integration:
//! - Taps into `AddFederationRuntimeDependencyHook` to collect dependencies
//! - Uses `JavascriptModulesRenderStartup` to inject startup calls when needed
//! - Integrates with compilation lifecycle hooks for runtime module management

use std::{
  collections::HashSet,
  sync::{Arc, Mutex},
};

use rspack_core::{
  ApplyContext, ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements,
  CompilationParams, CompilationRuntimeRequirementInTree, CompilerCompilation, CompilerOptions,
  DependencyId, ModuleIdentifier, Plugin, PluginContext, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{JavascriptModulesRenderStartup, JsPlugin, RenderSource};
use rspack_sources::{ConcatSource, RawStringSource, SourceExt};

use super::{
  embed_federation_runtime_module::{
    EmbedFederationRuntimeModule, EmbedFederationRuntimeModuleOptions,
  },
  federation_modules_plugin::{AddFederationRuntimeDependencyHook, FederationModulesPlugin},
  federation_runtime_dependency::FederationRuntimeDependency,
};

struct FederationRuntimeDependencyCollector {
  collected_dependency_ids: Arc<Mutex<HashSet<DependencyId>>>,
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
  collected_dependency_ids: Arc<Mutex<HashSet<DependencyId>>>,
}

impl EmbedFederationRuntimePlugin {
  pub fn new() -> Self {
    Self::new_inner(Arc::new(Mutex::new(HashSet::new())))
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

  // Skip build time chunks - these are development-only and don't need federation runtime
  if chunk.name() == Some("build time chunk") {
    return Ok(());
  }

  // Determine if this chunk needs federation runtime support
  let has_runtime = chunk.has_runtime(&compilation.chunk_group_by_ukey);
  let has_entry_modules = compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey)
    > 0;

  // Federation is enabled for:
  // 1. Runtime chunks (need to host the federation runtime module)
  // 2. Entry chunks (need startup calls for federation initialization)
  let is_enabled = has_runtime || has_entry_modules;

  if is_enabled {
    // Add STARTUP requirement to enable startup function creation
    // - For runtime chunks: enables creation of __webpack_require__.startup function that can be wrapped
    // - For entry chunks: signals that entry module execution should be wrapped in a callable function
    runtime_requirements.insert(RuntimeGlobals::STARTUP);
  }

  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for EmbedFederationRuntimePlugin)]
async fn runtime_requirement_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  _runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  // Skip build time chunks - these are development-only and don't need federation runtime
  if chunk.name() == Some("build time chunk") {
    return Ok(None);
  }

  // Only inject EmbedFederationRuntimeModule into runtime chunks
  // Runtime chunks are responsible for hosting the federation initialization logic
  let has_runtime = chunk.has_runtime(&compilation.chunk_group_by_ukey);
  if has_runtime {
    // Collect current snapshot of federation dependencies
    // This ensures the runtime module has access to all federation deps discovered so far
    let collected_ids_snapshot = self
      .collected_dependency_ids
      .lock()
      .expect("Failed to lock collected_dependency_ids")
      .iter()
      .cloned()
      .collect::<Vec<DependencyId>>();

    // Create runtime module options with collected dependencies
    let emro = EmbedFederationRuntimeModuleOptions {
      collected_dependency_ids: collected_ids_snapshot,
    };

    // Inject EmbedFederationRuntimeModule into this runtime chunk
    // This module will generate the "oldStartup wrapper" code that ensures
    // federation runtime modules execute before any other modules
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

  // Register the render startup hook
  let mut js_hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  js_hooks.render_startup.tap(render_startup::new(self));

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

  // Skip build time chunks - these are development-only and don't need federation runtime
  if chunk.name() == Some("build time chunk") {
    return Ok(());
  }

  // Only process chunks that have federation dependencies
  // No point in adding startup calls if there's no federation runtime to initialize
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

  // SCENARIO 1: Runtime chunks with entry modules (e.g., container chunks)
  // The JavaScript plugin already creates a startup function and calls it naturally.
  // Our EmbedFederationRuntimeModule will wrap the startup function, so we don't interfere.
  if has_runtime && has_entry_modules {
    return Ok(());
  }

  // SCENARIO 2: Entry chunks that delegate their runtime to other chunks
  // These chunks need explicit startup calls to trigger the startup function
  // in their delegated runtime chunks (where federation initialization happens).
  if !has_runtime && has_entry_modules {
    let mut startup_with_call = ConcatSource::default();

    // Add runtime startup call at the beginning to trigger startup function execution
    // This call will execute in the delegated runtime chunk where EmbedFederationRuntimeModule
    // has wrapped the startup function with federation runtime initialization
    startup_with_call.add(RawStringSource::from(
      "\n// Federation runtime initialization call\n",
    ));
    startup_with_call.add(RawStringSource::from(format!(
      "{}();\n",
      RuntimeGlobals::STARTUP.name()
    )));

    // Add the original startup source after the federation call
    startup_with_call.add(render_source.source.clone());

    render_source.source = startup_with_call.boxed();
  }

  // SCENARIO 3: Other chunk types (no runtime, no entry modules) are ignored
  // These don't need federation startup handling

  Ok(())
}

impl Plugin for EmbedFederationRuntimePlugin {
  fn name(&self) -> &'static str {
    "EmbedFederationRuntimePlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    ctx
      .context
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements_tree::new(self));
    ctx
      .context
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
