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
  pub fn new(normal_module_factory: Arc<NormalModuleFactory>) -> Self {
    Self {
      normal_module_factory,
    }
  }
}

#[async_trait::async_trait]
impl ModuleFactory for LazyCompilationDependencyFactory {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    let dep: &LazyCompilationDependency = data
      .dependency
      .as_any()
      .downcast_ref()
      .expect("should be lazy compile dependency");

    let proxy_data = &dep.original_module_create_data;

    let dep = dep.clone();

    let mut create_data = ModuleFactoryCreateData {
      resolve_options: proxy_data.resolve_options.clone(),
      context: proxy_data.context.clone(),
      dependency: Box::new(dep),
      issuer: proxy_data.issuer.clone(),
      issuer_identifier: proxy_data.issuer_identifier,
      file_dependencies: proxy_data.file_dependencies.clone(),
      context_dependencies: proxy_data.context_dependencies.clone(),
      missing_dependencies: proxy_data.missing_dependencies.clone(),
      diagnostics: proxy_data.diagnostics.clone(),
    };

    self.normal_module_factory.create(&mut create_data).await
  }
}
