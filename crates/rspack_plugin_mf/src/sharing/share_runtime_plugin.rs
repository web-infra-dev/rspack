use rspack_core::{
  ChunkUkey, Compilation, CompilationRuntimeRequirementInTree, Plugin, PluginContext,
  RuntimeGlobals, RuntimeModuleExt,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::ShareRuntimeModule;

#[plugin]
#[derive(Debug)]
pub struct ShareRuntimePlugin {
  enhanced: bool,
}

impl ShareRuntimePlugin {
  pub fn new(enhanced: bool) -> Self {
    Self::new_inner(enhanced)
  }
}

#[plugin_hook(CompilationRuntimeRequirementInTree for ShareRuntimePlugin)]
fn runtime_requirements_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &RuntimeGlobals,
  _runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  if runtime_requirements.contains(RuntimeGlobals::SHARE_SCOPE_MAP) {
    compilation.add_runtime_module(chunk_ukey, ShareRuntimeModule::new(self.enhanced).boxed())?;
  }
  Ok(None)
}

impl Plugin for ShareRuntimePlugin {
  fn name(&self) -> &'static str {
    "rspack.ShareRuntimePlugin"
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
    Ok(())
  }
}
