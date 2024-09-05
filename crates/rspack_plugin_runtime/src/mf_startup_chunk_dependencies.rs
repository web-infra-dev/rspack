use async_trait::async_trait;
use rspack_core::rspack_sources::ConcatSource;
use rspack_core::{
  ApplyContext, ChunkLoading, ChunkUkey, Compilation,
  CompilationAdditionalChunkRuntimeRequirements, CompilationAdditionalTreeRuntimeRequirements,
  CompilationParams, CompilationRuntimeRequirementInTree, CompilerCompilation, CompilerOptions,
  ModuleIdentifier, Plugin, PluginContext, RuntimeGlobals, RuntimeModuleExt,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::JsPlugin;
use rspack_plugin_javascript::{JavascriptModulesRenderStartup, RenderSource};

use crate::runtime_module::{
  is_enabled_for_chunk, StartupChunkDependenciesRuntimeModule, StartupEntrypointRuntimeModule,
};

#[plugin]
#[derive(Debug)]
pub struct MfStartupChunkDependenciesPlugin {
  chunk_loading: ChunkLoading,
  async_chunk_loading: bool,
}

impl MfStartupChunkDependenciesPlugin {
  pub fn new(chunk_loading: ChunkLoading, async_chunk_loading: bool) -> Self {
    Self::new_inner(chunk_loading, async_chunk_loading)
  }
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for MfStartupChunkDependenciesPlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &self.chunk_loading, compilation);
  if compilation
    .chunk_graph
    .has_chunk_entry_dependent_chunks(chunk_ukey, &compilation.chunk_group_by_ukey)
    && is_enabled_for_chunk
  {
    runtime_requirements.insert(RuntimeGlobals::STARTUP);
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES);
    compilation.add_runtime_module(
      chunk_ukey,
      StartupChunkDependenciesRuntimeModule::new(self.async_chunk_loading).boxed(),
    )?;
  }
  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for MfStartupChunkDependenciesPlugin)]
fn runtime_requirements_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &self.chunk_loading, compilation);

  if runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT) && is_enabled_for_chunk {
    runtime_requirements_mut.insert(RuntimeGlobals::REQUIRE);
    runtime_requirements_mut.insert(RuntimeGlobals::ENSURE_CHUNK);
    runtime_requirements_mut.insert(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES);
    compilation.add_runtime_module(
      chunk_ukey,
      StartupEntrypointRuntimeModule::new(self.async_chunk_loading).boxed(),
    )?;
  }

  Ok(None)
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for MfStartupChunkDependenciesPlugin)]
fn additional_chunk_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &self.chunk_loading, compilation);

  if is_enabled_for_chunk
    && compilation
      .chunk_graph
      .get_number_of_entry_modules(chunk_ukey)
      == 0
  {
    runtime_requirements.insert(RuntimeGlobals::FEDERATION_STARTUP);
  }

  Ok(())
}

#[plugin_hook(JavascriptModulesRenderStartup for MfStartupChunkDependenciesPlugin)]
fn render_startup(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _module: &ModuleIdentifier,
  render_source: &mut RenderSource,
) -> Result<()> {
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &self.chunk_loading, compilation);
  if !is_enabled_for_chunk {
    return Ok(());
  }

  let mut source = ConcatSource::default();
  source.add(render_source.source.clone());

  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let federation_runtime_module = compilation
    .chunk_graph
    .get_chunk_entry_modules(chunk_ukey)
    .iter();

  // if let Some(federation_runtime_module) = federation_runtime_module {
  //   let federation_module_id = compilation
  //     .chunk_graph
  //     .get_module_id(federation_runtime_module);
  //   source.add(RawSource::from(format!(
  //     "{}({});\n",
  //     RuntimeGlobals::REQUIRE,
  //     federation_module_id
  //   )));
  // }

  // render_source.source = source.boxed();
  Ok(())
}

#[plugin_hook(CompilerCompilation for MfStartupChunkDependenciesPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation);
  hooks.render_startup.tap(render_startup::new(self));
  // hooks.chunk_hash.tap(js_chunk_hash::new(self));
  Ok(())
}

#[async_trait]
impl Plugin for MfStartupChunkDependenciesPlugin {
  fn name(&self) -> &'static str {
    "MfStartupChunkDependenciesPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    ctx
      .context
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));
    ctx
      .context
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements::new(self));
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    Ok(())
  }
}
