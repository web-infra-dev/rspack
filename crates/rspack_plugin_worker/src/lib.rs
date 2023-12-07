use async_trait::async_trait;
use rspack_core::{
  CompilationArgs, CompilationParams, DependencyType, Plugin, PluginCompilationHookOutput,
};

#[derive(Debug)]
pub struct WorkerPlugin;

#[async_trait]
impl Plugin for WorkerPlugin {
  async fn compilation(
    &self,
    args: CompilationArgs<'_>,
    params: &CompilationParams,
  ) -> PluginCompilationHookOutput {
    args.compilation.set_dependency_factory(
      DependencyType::NewWorker,
      params.normal_module_factory.clone(),
    );
    Ok(())
  }
}
