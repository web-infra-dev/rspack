//! # ModuleFederationRuntimePlugin
//!
//! Main orchestration plugin for Module Federation runtime functionality.
//! Coordinates federation plugins, manages runtime dependencies, and adds the base FederationRuntimeModule.

use async_trait::async_trait;
use rspack_core::{
  ApplyContext, BoxDependency, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilerFinishMake, CompilerOptions, EntryOptions,
  Plugin, PluginContext, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use serde::Deserialize;

use super::{
  embed_federation_runtime_plugin::EmbedFederationRuntimePlugin,
  federation_data_runtime_module::FederationDataRuntimeModule,
  federation_modules_plugin::FederationModulesPlugin,
  federation_runtime_dependency::FederationRuntimeDependency,
  hoist_container_references_plugin::HoistContainerReferencesPlugin,
};

#[derive(Debug, Default, Deserialize, Clone)]
pub struct ModuleFederationRuntimePluginOptions {
  pub entry_runtime: Option<String>,
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
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  _runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  // Add base FederationRuntimeModule which is responsible for providing bundler data to the runtime.
  compilation.add_runtime_module(chunk_ukey, Box::<FederationDataRuntimeModule>::default())?;

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

#[async_trait]
impl Plugin for ModuleFederationRuntimePlugin {
  fn name(&self) -> &'static str {
    "rspack.container.ModuleFederationRuntimePlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));

    ctx
      .context
      .compiler_hooks
      .finish_make
      .tap(finish_make::new(self));

    // Apply supporting plugins
    EmbedFederationRuntimePlugin::default()
      .apply(PluginContext::with_context(ctx.context), options)?;
    HoistContainerReferencesPlugin::default()
      .apply(PluginContext::with_context(ctx.context), options)?;

    Ok(())
  }
}
