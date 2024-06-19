use rspack_core::{
  ChunkUkey, Compilation, CompilationAdditionalTreeRuntimeRequirements,
  CompilationRuntimeRequirementInTree, Plugin, PluginContext, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::runtime_module::RspackExposeGlobalRuntimeModule;

#[plugin]
#[derive(Debug)]
pub struct ExposeGlobalPlugin {
  global: String,
}

impl ExposeGlobalPlugin {
  pub fn new(global: String) -> Self {
    Self::new_inner(global)
  }
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for ExposeGlobalPlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  _chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  runtime_requirements.insert(RuntimeGlobals::GLOBAL);
  if compilation.options.output.unique_name.is_empty() {
    runtime_requirements.insert(RuntimeGlobals::GET_FULL_HASH);
  }
  runtime_requirements.insert(RuntimeGlobals::RSPACK_EXPOSE_GLOBAL);
  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for ExposeGlobalPlugin)]
fn runtime_requirements_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &RuntimeGlobals,
  _runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  if runtime_requirements.contains(RuntimeGlobals::RSPACK_EXPOSE_GLOBAL) {
    compilation.add_runtime_module(
      chunk_ukey,
      Box::new(RspackExposeGlobalRuntimeModule::new(self.global.clone())),
    )?;
  }
  Ok(None)
}

impl Plugin for ExposeGlobalPlugin {
  fn name(&self) -> &'static str {
    "ExposeGlobalPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut rspack_core::ApplyContext>,
    _options: &mut rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));
    ctx
      .context
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    Ok(())
  }
}
