use rspack_core::{
  CircularModulesInfo, Compilation, CompilationOptimizeModules, CompilerMake, Plugin,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};

#[plugin]
#[derive(Debug, Default)]
pub struct CircularModulesInfoPlugin;

#[plugin_hook(CompilerMake for CircularModulesInfoPlugin)]
async fn make(&self, compilation: &mut Compilation) -> Result<()> {
  compilation.circular_modules.enable_collect_modules();
  Ok(())
}

#[plugin_hook(CompilationOptimizeModules for CircularModulesInfoPlugin)]
async fn optimize_modules(
  &self,
  compilation: &Compilation,
  circular_modules: &mut CircularModulesInfo,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<Option<bool>> {
  circular_modules.ensure_circular_modules_info(compilation);
  Ok(None)
}

impl Plugin for CircularModulesInfoPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.make.tap(make::new(self));
    ctx
      .compilation_hooks
      .optimize_modules
      .tap(optimize_modules::new(self));
    Ok(())
  }
}
