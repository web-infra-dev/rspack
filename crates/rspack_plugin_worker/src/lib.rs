use async_trait::async_trait;
use rspack_core::{
  ApplyContext, Compilation, CompilationParams, CompilerOptions, DependencyType, PluginContext,
};
use rspack_error::Result;
use rspack_hook::AsyncSeries2;

#[derive(Debug)]
pub struct WorkerPlugin;

struct WorkerPluginCompilationHook;

#[async_trait]
impl AsyncSeries2<Compilation, CompilationParams> for WorkerPluginCompilationHook {
  async fn run(&self, compilation: &mut Compilation, params: &mut CompilationParams) -> Result<()> {
    compilation.set_dependency_factory(
      DependencyType::NewWorker,
      params.normal_module_factory.clone(),
    );
    Ok(())
  }
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
      .tap(Box::new(WorkerPluginCompilationHook));
    Ok(())
  }
}
