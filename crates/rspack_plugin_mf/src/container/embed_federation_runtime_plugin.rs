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

  // Skip build time chunks
  if chunk.name() == Some("build time chunk") {
    return Ok(());
  }

  // Add STARTUP requirement to runtime chunks OR application chunks with entry modules
  let has_runtime = chunk.has_runtime(&compilation.chunk_group_by_ukey);
  let has_entry_modules = compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey)
    > 0;
  let is_enabled = has_runtime || has_entry_modules;

  if is_enabled {
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

  // Skip build time chunks
  if chunk.name() == Some("build time chunk") {
    return Ok(None);
  }

  // Only add EmbedFederationRuntimeModule to runtime chunks
  let has_runtime = chunk.has_runtime(&compilation.chunk_group_by_ukey);
  if has_runtime {
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

  // Skip build time chunks
  if chunk.name() == Some("build time chunk") {
    return Ok(());
  }

  // Check if this chunk needs federation runtime initialization
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

  // For chunks with both runtime and entry modules (like container chunk):
  // The JavaScript plugin already handles the startup call in its render_startup logic.
  // We should not interfere.
  if has_runtime && has_entry_modules {
    return Ok(());
  }

  // For entry chunks that delegate their runtime to other chunks:
  // These chunks need the startup call to ensure federation runtime gets initialized
  // in the delegated runtime chunk.
  if !has_runtime && has_entry_modules {
    let mut startup_with_call = ConcatSource::default();

    // Add runtime startup call at the beginning to ensure federation initialization
    startup_with_call.add(RawStringSource::from(
      "\n// Federation runtime initialization call\n",
    ));
    startup_with_call.add(RawStringSource::from(format!(
      "{}();\n",
      RuntimeGlobals::STARTUP.name()
    )));

    // Add the original startup source
    startup_with_call.add(render_source.source.clone());

    render_source.source = startup_with_call.boxed();
  }

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
