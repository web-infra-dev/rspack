use rspack_core::{
  ApplyContext, Compilation, CompilationParams, CompilerCompilation, CompilerOptions,
  DependencyType, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

#[plugin]
#[derive(Debug, Default)]
pub struct WorkerPlugin;

#[plugin_hook(CompilerCompilation for WorkerPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(
    DependencyType::NewWorker,
    params.normal_module_factory.clone(),
  );
  Ok(())
}

impl rspack_core::Plugin for WorkerPlugin {
  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    Ok(())
  }
}
