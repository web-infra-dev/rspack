use std::sync::Arc;

use rspack_core::{
  ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, NormalModuleFactory,
};
use rspack_error::Result;

use crate::dependency::LazyCompilationDependency;

#[derive(Debug)]
pub(crate) struct LazyCompilationDependencyFactory {
  normal_module_factory: Arc<NormalModuleFactory>,
}

impl LazyCompilationDependencyFactory {
  pub(crate) fn new(normal_module_factory: Arc<NormalModuleFactory>) -> Self {
    Self {
      normal_module_factory,
    }
  }
}

#[async_trait::async_trait]
impl ModuleFactory for LazyCompilationDependencyFactory {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    let dep: &LazyCompilationDependency = data.dependencies[0]
      .as_any()
      .downcast_ref()
      .expect("should be lazy compile dependency");
    let options = dep.options();

    data
      .file_dependencies
      .extend(options.file_dependencies.clone());
    data
      .context_dependencies
      .extend(options.context_dependencies.clone());
    data
      .missing_dependencies
      .extend(options.missing_dependencies.clone());
    data.diagnostics.extend(options.diagnostics.clone());

    self.normal_module_factory.create(data).await
  }
}
