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
    let dep: &LazyCompilationDependency = data.dependencies[0]
      .as_any()
      .downcast_ref()
      .expect("should be lazy compile dependency");

    let proxy_data = &dep.original_module_create_data;

    let dep = dep.clone();

    let mut create_data = ModuleFactoryCreateData {
      compiler_id: data.compiler_id,
      compilation_id: data.compilation_id,
      resolve_options: proxy_data.resolve_options.clone(),
      options: data.options.clone(),
      context: proxy_data.context.clone(),
      dependencies: vec![Box::new(dep)],
      issuer: proxy_data.issuer.clone(),
      issuer_layer: proxy_data.issuer_layer.clone(),
      issuer_identifier: proxy_data.issuer_identifier,
      resolver_factory: proxy_data.resolver_factory.clone(),
      file_dependencies: proxy_data.file_dependencies.clone(),
      context_dependencies: proxy_data.context_dependencies.clone(),
      missing_dependencies: proxy_data.missing_dependencies.clone(),
      diagnostics: proxy_data.diagnostics.clone(),
    };

    self.normal_module_factory.create(&mut create_data).await
  }
}
