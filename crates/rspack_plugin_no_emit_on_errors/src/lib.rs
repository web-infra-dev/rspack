use std::fmt::Debug;

use rspack_core::{
  ApplyContext, Compilation, CompilerOptions, CompilerShouldEmit, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

#[plugin]
#[derive(Debug)]
pub struct NoEmitOnErrorsPlugin {}

impl NoEmitOnErrorsPlugin {
  pub fn new() -> Self {
    Self::new_inner()
  }
}

#[plugin_hook(CompilerShouldEmit for NoEmitOnErrorsPlugin)]
async fn should_emit(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  Ok(Some(compilation.get_errors().next().is_none()))
}

impl Plugin for NoEmitOnErrorsPlugin {
  fn name(&self) -> &'static str {
    "NoEmitOnErrorsPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .should_emit
      .tap(should_emit::new(self));
    Ok(())
  }
}
