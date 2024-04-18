use std::sync::Arc;

use async_trait::async_trait;
use rspack_core::{
  ApplyContext, ChunkUkey, Compilation, CompilationAdditionalTreeRuntimeRequirements,
  CompilationParams, CompilerCompilation, CompilerOptions, Plugin, PluginContext, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use serde::Serialize;

use super::federation_module::FederationRuntimeModule;

#[derive(Debug, Clone, Serialize)]
pub struct RemoteOptions {
  pub external: Vec<String>,
  pub share_scope: String,
}

pub type Remotes = Vec<(String, RemoteOptions)>;

#[derive(Debug, Serialize)]
pub struct FederationRuntimePluginOptions {
  pub name: Option<String>,
  pub remotes: Option<Remotes>,
}

#[plugin]
#[derive(Debug)]
pub struct FederationRuntimePlugin {
  options: FederationRuntimePluginOptions,
}
impl FederationRuntimePlugin {
  pub fn new(options: FederationRuntimePluginOptions) -> Self {
    Self::new_inner(options)
  }

  fn constructor_name(&self) -> &'static str {
    "FederationRuntimePlugin"
  }
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for FederationRuntimePlugin)]
fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let federation_global = format!("{}.federation", RuntimeGlobals::REQUIRE);
  if !runtime_requirements.contains(&federation_global) {
    runtime_requirements_mut.insert(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION);
    runtime_requirements_mut.insert(RuntimeGlobals::MODULE_CACHE);
    runtime_requirements_mut.insert(RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT);
    runtime_requirements_mut.insert(federation_global.clone());
    compilation.add_runtime_module(
      chunk_ukey,
      Box::new(FederationRuntimeModule::new(
        runtime_requirements.clone(),
        self.options.name.clone(),
        (&self.options).clone(),
      )),
    )?;
  }
  Ok(None)
}

#[async_trait]
impl Plugin for FederationRuntimePlugin {
  fn name(&self) -> &'static str {
    "rspack.FederationRuntimePlugin"
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
    Ok(())
  }
}
