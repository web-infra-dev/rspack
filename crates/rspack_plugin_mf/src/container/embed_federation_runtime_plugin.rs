use std::{
  collections::HashSet,
  sync::{Arc, Mutex},
};

use rspack_core::{
  ApplyContext, ChunkGraph, ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements,
  CompilationAdditionalTreeRuntimeRequirements, CompilationParams,
  CompilationRuntimeRequirementInTree, CompilerCompilation, CompilerOptions, DependencyId,
  ModuleIdentifier, Plugin, PluginContext, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{JavascriptModulesRenderStartup, JsPlugin, RenderSource};
use rspack_sources::{ConcatSource, RawStringSource, SourceExt};

use super::{
  container_entry_module::ContainerEntryModule,
  embed_federation_runtime_module::{
    EmbedFederationRuntimeModule, EmbedFederationRuntimeModuleOptions,
  },
  federation_modules_plugin::{AddFederationRuntimeDependencyHook, FederationModulesPlugin},
  federation_runtime_dependency::FederationRuntimeDependency,
};

#[derive(Debug, Default)]
struct EmbedFederationRuntimePluginOptions {
  // Currently no options, but can be extended later
}

struct FederationRuntimeDependencyCollector {
  collected_dependency_ids: Arc<Mutex<HashSet<DependencyId>>>,
}

// Helper function to decode RuntimeGlobals
fn debug_runtime_globals(globals: &RuntimeGlobals) -> String {
  let mut result = String::new();

  // Define all flag constants with their bit positions
  let flags = [
    (RuntimeGlobals::REQUIRE_SCOPE, "REQUIRE_SCOPE"),
    (RuntimeGlobals::MODULE, "MODULE"),
    (RuntimeGlobals::MODULE_ID, "MODULE_ID"),
    (RuntimeGlobals::REQUIRE, "REQUIRE"),
    (RuntimeGlobals::MODULE_CACHE, "MODULE_CACHE"),
    (RuntimeGlobals::ENSURE_CHUNK, "ENSURE_CHUNK"),
    (
      RuntimeGlobals::ENSURE_CHUNK_HANDLERS,
      "ENSURE_CHUNK_HANDLERS",
    ),
    (RuntimeGlobals::PUBLIC_PATH, "PUBLIC_PATH"),
    (
      RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME,
      "GET_CHUNK_SCRIPT_FILENAME",
    ),
    (
      RuntimeGlobals::GET_CHUNK_CSS_FILENAME,
      "GET_CHUNK_CSS_FILENAME",
    ),
    (RuntimeGlobals::LOAD_SCRIPT, "LOAD_SCRIPT"),
    (RuntimeGlobals::HAS_OWN_PROPERTY, "HAS_OWN_PROPERTY"),
    (
      RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY,
      "MODULE_FACTORIES_ADD_ONLY",
    ),
    (RuntimeGlobals::ON_CHUNKS_LOADED, "ON_CHUNKS_LOADED"),
    (RuntimeGlobals::CHUNK_CALLBACK, "CHUNK_CALLBACK"),
    (RuntimeGlobals::MODULE_FACTORIES, "MODULE_FACTORIES"),
    (
      RuntimeGlobals::INTERCEPT_MODULE_EXECUTION,
      "INTERCEPT_MODULE_EXECUTION",
    ),
    (
      RuntimeGlobals::HMR_DOWNLOAD_MANIFEST,
      "HMR_DOWNLOAD_MANIFEST",
    ),
    (
      RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS,
      "HMR_DOWNLOAD_UPDATE_HANDLERS",
    ),
    (
      RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME,
      "GET_UPDATE_MANIFEST_FILENAME",
    ),
    (
      RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME,
      "GET_CHUNK_UPDATE_SCRIPT_FILENAME",
    ),
    (
      RuntimeGlobals::GET_CHUNK_UPDATE_CSS_FILENAME,
      "GET_CHUNK_UPDATE_CSS_FILENAME",
    ),
    (RuntimeGlobals::HMR_MODULE_DATA, "HMR_MODULE_DATA"),
    (
      RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX,
      "HMR_RUNTIME_STATE_PREFIX",
    ),
    (
      RuntimeGlobals::EXTERNAL_INSTALL_CHUNK,
      "EXTERNAL_INSTALL_CHUNK",
    ),
    (RuntimeGlobals::GET_FULL_HASH, "GET_FULL_HASH"),
    (RuntimeGlobals::GLOBAL, "GLOBAL"),
    (
      RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME,
      "RETURN_EXPORTS_FROM_RUNTIME",
    ),
    (RuntimeGlobals::INSTANTIATE_WASM, "INSTANTIATE_WASM"),
    (RuntimeGlobals::ASYNC_MODULE, "ASYNC_MODULE"),
    (RuntimeGlobals::BASE_URI, "BASE_URI"),
    (RuntimeGlobals::MODULE_LOADED, "MODULE_LOADED"),
    (RuntimeGlobals::STARTUP_ENTRYPOINT, "STARTUP_ENTRYPOINT"),
    (RuntimeGlobals::CREATE_SCRIPT_URL, "CREATE_SCRIPT_URL"),
    (RuntimeGlobals::CREATE_SCRIPT, "CREATE_SCRIPT"),
    (
      RuntimeGlobals::GET_TRUSTED_TYPES_POLICY,
      "GET_TRUSTED_TYPES_POLICY",
    ),
    (
      RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
      "DEFINE_PROPERTY_GETTERS",
    ),
    (RuntimeGlobals::ENTRY_MODULE_ID, "ENTRY_MODULE_ID"),
    (RuntimeGlobals::STARTUP_NO_DEFAULT, "STARTUP_NO_DEFAULT"),
    (
      RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES,
      "ENSURE_CHUNK_INCLUDE_ENTRIES",
    ),
    (RuntimeGlobals::STARTUP, "STARTUP"),
    (
      RuntimeGlobals::MAKE_NAMESPACE_OBJECT,
      "MAKE_NAMESPACE_OBJECT",
    ),
    (RuntimeGlobals::EXPORTS, "EXPORTS"),
    (
      RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT,
      "COMPAT_GET_DEFAULT_EXPORT",
    ),
    (
      RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
      "CREATE_FAKE_NAMESPACE_OBJECT",
    ),
    (
      RuntimeGlobals::NODE_MODULE_DECORATOR,
      "NODE_MODULE_DECORATOR",
    ),
    (RuntimeGlobals::ESM_MODULE_DECORATOR, "ESM_MODULE_DECORATOR"),
    (RuntimeGlobals::SYSTEM_CONTEXT, "SYSTEM_CONTEXT"),
    (RuntimeGlobals::THIS_AS_EXPORTS, "THIS_AS_EXPORTS"),
    (
      RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE,
      "CURRENT_REMOTE_GET_SCOPE",
    ),
    (RuntimeGlobals::SHARE_SCOPE_MAP, "SHARE_SCOPE_MAP"),
    (RuntimeGlobals::INITIALIZE_SHARING, "INITIALIZE_SHARING"),
    (RuntimeGlobals::SCRIPT_NONCE, "SCRIPT_NONCE"),
    (RuntimeGlobals::RELATIVE_URL, "RELATIVE_URL"),
    (RuntimeGlobals::CHUNK_NAME, "CHUNK_NAME"),
    (RuntimeGlobals::RUNTIME_ID, "RUNTIME_ID"),
    (RuntimeGlobals::PREFETCH_CHUNK, "PREFETCH_CHUNK"),
    (
      RuntimeGlobals::PREFETCH_CHUNK_HANDLERS,
      "PREFETCH_CHUNK_HANDLERS",
    ),
    (RuntimeGlobals::PRELOAD_CHUNK, "PRELOAD_CHUNK"),
    (
      RuntimeGlobals::PRELOAD_CHUNK_HANDLERS,
      "PRELOAD_CHUNK_HANDLERS",
    ),
    (
      RuntimeGlobals::UNCAUGHT_ERROR_HANDLER,
      "UNCAUGHT_ERROR_HANDLER",
    ),
    (RuntimeGlobals::RSPACK_VERSION, "RSPACK_VERSION"),
    (RuntimeGlobals::HAS_CSS_MODULES, "HAS_CSS_MODULES"),
    (RuntimeGlobals::RSPACK_UNIQUE_ID, "RSPACK_UNIQUE_ID"),
    (RuntimeGlobals::HAS_FETCH_PRIORITY, "HAS_FETCH_PRIORITY"),
    (RuntimeGlobals::AMD_DEFINE, "AMD_DEFINE"),
    (RuntimeGlobals::AMD_OPTIONS, "AMD_OPTIONS"),
  ];

  let mut flag_count = 0;
  for (flag, name) in flags.iter() {
    if globals.contains(*flag) {
      result.push_str(&format!("  - {} ({})\n", name, flag.name()));
      flag_count += 1;
    }
  }

  if flag_count == 0 {
    result.push_str("  (No flags set)\n");
  }

  result
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
  #[allow(dead_code)]
  options: EmbedFederationRuntimePluginOptions,
  collected_dependency_ids: Arc<Mutex<HashSet<DependencyId>>>,
}

impl EmbedFederationRuntimePlugin {
  pub fn new() -> Self {
    Self::new_inner(
      EmbedFederationRuntimePluginOptions::default(),
      Arc::new(Mutex::new(HashSet::new())),
    )
  }
}

// Helper to check if a chunk is an entrypoint and not a special chunk
fn is_enabled_for_chunk(compilation: &Compilation, chunk_ukey: &ChunkUkey) -> bool {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  if chunk.name() == Some("build time chunk") {
    return false;
  }

  // Get entry modules for the chunk
  let entry_modules = compilation.chunk_graph.get_chunk_entry_modules(chunk_ukey);

  // Find the last entry module (like .reverse()[0] in JS)
  if let Some(module_id) = entry_modules.last() {
    let module_graph = compilation.get_module_graph();
    if let Some(module) = module_graph.module_by_identifier(module_id) {
      if module.as_any().is::<ContainerEntryModule>() {
        return false;
      }
    }
  }

  true
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for EmbedFederationRuntimePlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  if is_enabled_for_chunk(compilation, chunk_ukey) {
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
    if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
      dbg!("Adding STARTUP_ENTRYPOINT as tree requirement", chunk_ukey);
      runtime_requirements.insert(RuntimeGlobals::STARTUP_ENTRYPOINT);
      runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
      runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES);
      let readable = debug_runtime_globals(runtime_requirements);
      dbg!("Current tree runtime_requirements (decoded)", &readable);
    }
  }
  Ok(())
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for EmbedFederationRuntimePlugin)]
async fn additional_chunk_runtime_requirements_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
    dbg!("Adding STARTUP to runtime chunk", chunk_ukey);
    runtime_requirements.insert(RuntimeGlobals::STARTUP);
    let readable = debug_runtime_globals(runtime_requirements);
    dbg!("Current chunk runtime_requirements (decoded)", &readable);
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
  if is_enabled_for_chunk(compilation, chunk_ukey) {
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
    if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
      dbg!(
        "Injecting EmbedFederationRuntimeModule (loosened condition)",
        chunk_ukey
      );
      let collected_ids_snapshot = self
        .collected_dependency_ids
        .lock()
        .unwrap()
        .iter()
        .cloned()
        .collect::<Vec<DependencyId>>();
      let emro = EmbedFederationRuntimeModuleOptions {
        collected_dependency_ids: collected_ids_snapshot,
      };
      compilation.add_runtime_module(
        chunk_ukey,
        Box::new(EmbedFederationRuntimeModule::new(emro)),
      )?;
      dbg!("EmbedFederationRuntimeModule injected", chunk_ukey);
    }
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
  dbg!("render_startup called", chunk_ukey);
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let entry_module_count = compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey);
  dbg!("entry_module_count", chunk_ukey, entry_module_count);
  if entry_module_count == 0 {
    return Ok(());
  }
  let tree_runtime_requirements =
    ChunkGraph::get_tree_runtime_requirements(compilation, chunk_ukey);
  dbg!(
    "tree_runtime_requirements",
    chunk_ukey,
    &tree_runtime_requirements
  );
  if tree_runtime_requirements.contains(RuntimeGlobals::STARTUP)
    || tree_runtime_requirements.contains(RuntimeGlobals::STARTUP_NO_DEFAULT)
  {
    dbg!("startup already present, skipping append", chunk_ukey);
    return Ok(());
  }
  dbg!(
    "Appending __webpack_require__.x() to entry chunk",
    chunk_ukey,
    entry_module_count
  );
  let mut startup_with_call = ConcatSource::default();

  startup_with_call.add(RawStringSource::from(
    "\n// Entrypoint: appended startup call because none was added automatically\n",
  ));
  startup_with_call.add(RawStringSource::from("__webpack_require__.x();\n"));
  startup_with_call.add(render_source.source.clone());
  render_source.source = startup_with_call.boxed();
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
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
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
