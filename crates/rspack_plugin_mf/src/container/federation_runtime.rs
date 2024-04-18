use async_trait::async_trait;
use rspack_core::{
  ApplyContext, ChunkUkey, Compilation, CompilationAdditionalTreeRuntimeRequirements,
  CompilerOptions, Plugin, PluginContext, RuntimeGlobals,
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
#[derive(Debug, Default)]
pub struct FederationRuntimePlugin;

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for FederationRuntimePlugin)]
fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  _runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  compilation.add_runtime_module(chunk_ukey, Box::new(FederationRuntimeModule::default()))?;
  Ok(())
}

#[async_trait]
impl Plugin for FederationRuntimePlugin {
  fn name(&self) -> &'static str {
    "rspack.container.FederationRuntimePlugin"
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
