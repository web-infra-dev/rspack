use rspack_core::{
  ChunkUkey, Compilation, CompilationRuntimeRequirementInTree, Plugin, RuntimeGlobals,
  RuntimeModule, RuntimeModuleExt,
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
async fn runtime_requirements_in_tree(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  _runtime_requirements_mut: &mut RuntimeGlobals,
  runtime_modules_to_add: &mut Vec<(ChunkUkey, Box<dyn RuntimeModule>)>,
) -> Result<Option<()>> {
  if runtime_requirements.contains(RuntimeGlobals::SHARE_SCOPE_MAP) {
    runtime_modules_to_add.push((
      *chunk_ukey,
      ShareRuntimeModule::new(&compilation.runtime_template, self.enhanced).boxed(),
    ));
  }
  Ok(None)
}

impl Plugin for ShareRuntimePlugin {
  fn name(&self) -> &'static str {
    "rspack.ShareRuntimePlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));
    Ok(())
  }
}
