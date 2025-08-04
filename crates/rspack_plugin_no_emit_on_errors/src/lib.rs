use std::fmt::Debug;

use rspack_core::{Compilation, CompilerShouldEmit, Plugin};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

#[plugin]
#[derive(Debug)]
pub struct NoEmitOnErrorsPlugin {}

impl Default for NoEmitOnErrorsPlugin {
  fn default() -> Self {
    Self::new_inner()
  }
}

#[plugin_hook(CompilerShouldEmit for NoEmitOnErrorsPlugin)]
async fn should_emit(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  if compilation.get_errors().next().is_some() {
    Ok(Some(false))
  } else {
    Ok(None)
  }
}

impl Plugin for NoEmitOnErrorsPlugin {
  fn name(&self) -> &'static str {
    "NoEmitOnErrorsPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.should_emit.tap(should_emit::new(self));
    Ok(())
  }
}
