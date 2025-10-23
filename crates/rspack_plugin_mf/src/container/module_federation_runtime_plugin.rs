//! # ModuleFederationRuntimePlugin
//!
//! Main orchestration plugin for Module Federation runtime functionality.
//! Coordinates federation plugins, manages runtime dependencies, and adds the base FederationRuntimeModule.

use std::sync::{Arc, Mutex};

use rspack_core::{
  BoxDependency, ChunkUkey, Compilation, CompilationAdditionalTreeRuntimeRequirements,
  CompilerFinishMake, DependencyId, EntryOptions, Plugin, RuntimeGlobals, RuntimeModuleExt,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use serde::Deserialize;

use super::{
  embed_federation_runtime_plugin::EmbedFederationRuntimePlugin,
  entry_runtime_module::EntryFederationRuntimeModule,
  federation_data_runtime_module::FederationDataRuntimeModule,
  federation_modules_plugin::FederationModulesPlugin,
  federation_runtime_dependency::FederationRuntimeDependency,
  hoist_container_references_plugin::HoistContainerReferencesPlugin,
};

#[derive(Debug, Default, Deserialize, Clone)]
pub struct ModuleFederationRuntimePluginOptions {
  pub entry_runtime: Option<String>,
  pub async_startup: Option<bool>,
}

#[plugin]
#[derive(Debug)]
pub struct ModuleFederationRuntimePlugin {
  options: ModuleFederationRuntimePluginOptions,
  entry_runtime_dependency: Arc<Mutex<Option<DependencyId>>>,
}

impl ModuleFederationRuntimePlugin {
  pub fn new(options: ModuleFederationRuntimePluginOptions) -> Self {
    Self::new_inner(options, Arc::new(Mutex::new(None)))
  }
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for ModuleFederationRuntimePlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  // Add base FederationRuntimeModule which is responsible for providing bundler data to the runtime.
  let entry_dep = {
    let guard = self.entry_runtime_dependency.lock().expect("lock poisoned");
    *guard
  };
  compilation.add_runtime_module(chunk_ukey, FederationDataRuntimeModule::default().boxed())?;

  let async_startup_enabled = compilation.options.experiments.mf_async_startup;

  if async_startup_enabled {
    if let Some(dep_id) = entry_dep {
      runtime_requirements.insert(RuntimeGlobals::REQUIRE);
      compilation.add_runtime_module(
        chunk_ukey,
        EntryFederationRuntimeModule::new(dep_id).boxed(),
      )?;
    }
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
  }

  Ok(())
}

#[plugin_hook(CompilerFinishMake for ModuleFederationRuntimePlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  // Only add the federation runtime entry if we have an entry_runtime configured
  // The entry_runtime should always be provided by ModuleFederationPlugin
  if let Some(entry_request) = self.options.entry_runtime.clone() {
    let federation_runtime_dep = FederationRuntimeDependency::new(entry_request.clone());

    let hooks = FederationModulesPlugin::get_compilation_hooks(compilation);

    hooks
      .add_federation_runtime_dependency
      .lock()
      .await
      .call(&federation_runtime_dep)
      .await?;

    // Store the dependency ID for later use in additional_tree_runtime_requirements
    {
      let mut guard = self
        .entry_runtime_dependency
        .lock()
        .expect("Failed to lock entry runtime dependency");
      *guard = Some(federation_runtime_dep.id);
    }

    let boxed_dep: BoxDependency = Box::new(federation_runtime_dep);
    let entry_options = EntryOptions::default();
    let args = vec![(boxed_dep, entry_options)];

    compilation.add_include(args).await?;
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

    ctx.compiler_hooks.finish_make.tap(finish_make::new(self));

    // Apply supporting plugins
    // Note: EmbedFederationRuntimePlugin will check async startup flag in its hooks
    EmbedFederationRuntimePlugin::default().apply(ctx)?;
    HoistContainerReferencesPlugin::default().apply(ctx)?;

    Ok(())
  }
}
