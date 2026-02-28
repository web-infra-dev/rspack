//! # ModuleFederationRuntimePlugin
//!
//! Main orchestration plugin for Module Federation runtime functionality.
//! Coordinates federation plugins, manages runtime dependencies, and adds the base FederationRuntimeModule.

use rspack_cacheable::cacheable;
use rspack_core::{
  AsyncModulesArtifact, BoxDependency, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilationFinishModules, CompilerFinishMake,
  EntryOptions, ExportsInfoArtifact, Plugin, RuntimeGlobals, RuntimeModule,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use serde::Deserialize;

use super::{
  container_entry_module::ContainerEntryModule,
  embed_federation_runtime_plugin::EmbedFederationRuntimePlugin,
  federation_data_runtime_module::FederationDataRuntimeModule,
  federation_modules_plugin::FederationModulesPlugin,
  federation_runtime_dependency::FederationRuntimeDependency,
  hoist_container_references_plugin::HoistContainerReferencesPlugin,
};

#[derive(Debug, Default, Deserialize, Clone)]
pub struct ModuleFederationRuntimePluginOptions {
  pub entry_runtime: Option<String>,
  #[serde(default)]
  pub experiments: ModuleFederationRuntimeExperimentsOptions,
}

#[cacheable]
#[derive(Debug, Default, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct ModuleFederationRuntimeExperimentsOptions {
  #[serde(default)]
  pub async_startup: bool,
}

#[plugin]
#[derive(Debug)]
pub struct ModuleFederationRuntimePlugin {
  options: ModuleFederationRuntimePluginOptions,
}

impl ModuleFederationRuntimePlugin {
  pub fn new(options: ModuleFederationRuntimePluginOptions) -> Self {
    Self::new_inner(options)
  }
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for ModuleFederationRuntimePlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &Compilation,
  _chunk_ukey: &ChunkUkey,
  _runtime_requirements: &mut RuntimeGlobals,
  runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  // Add base FederationRuntimeModule which is responsible for providing bundler data to the runtime.
  runtime_modules.push(Box::new(FederationDataRuntimeModule::new(
    &compilation.runtime_template,
  )));

  Ok(())
}

#[plugin_hook(CompilerFinishMake for ModuleFederationRuntimePlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  if let Some(entry_request) = self.options.entry_runtime.clone() {
    let federation_runtime_dep = FederationRuntimeDependency::new(entry_request.clone());

    let hooks = FederationModulesPlugin::get_compilation_hooks(compilation);

    hooks
      .add_federation_runtime_dependency
      .lock()
      .await
      .call(&federation_runtime_dep)
      .await?;

    let boxed_dep: BoxDependency = Box::new(federation_runtime_dep);
    let entry_options = EntryOptions::default();
    let args = vec![(boxed_dep, entry_options)];

    compilation.add_include(args).await?;
  }

  Ok(())
}

// When MF async startup is enabled, STARTUP may resolve asynchronously even if the entry module
// itself doesn't have top-level await. Mark MF container entry modules as async so downstream
// renderers (e.g. library wrappers) can safely `await` exports without sprinkling MF-specific
// RuntimeGlobals checks across generic plugins.
#[plugin_hook(CompilationFinishModules for ModuleFederationRuntimePlugin, stage = 1000)]
async fn finish_modules(
  &self,
  compilation: &Compilation,
  async_modules_artifact: &mut AsyncModulesArtifact,
  _exports_info_artifact: &mut ExportsInfoArtifact,
) -> Result<()> {
  if !self.options.experiments.async_startup {
    return Ok(());
  }

  let module_graph = compilation.get_module_graph();
  for (module_identifier, module) in module_graph.modules() {
    if module
      .as_ref()
      .as_any()
      .downcast_ref::<ContainerEntryModule>()
      .is_some()
    {
      async_modules_artifact.insert(*module_identifier);
    }
  }

  Ok(())
}

impl Plugin for ModuleFederationRuntimePlugin {
  fn name(&self) -> &'static str {
    "rspack.container.ModuleFederationRuntimePlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));

    ctx
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));

    ctx.compiler_hooks.finish_make.tap(finish_make::new(self));

    // Apply supporting plugins
    EmbedFederationRuntimePlugin::new(self.options.experiments.clone()).apply(ctx)?;
    HoistContainerReferencesPlugin::default().apply(ctx)?;

    Ok(())
  }
}
