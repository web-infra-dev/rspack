use std::sync::Arc;

use rspack_core::{
  ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, NormalModuleFactory,
};
use rspack_error::Result;

use crate::dependency::{LazyCompilationDependency, ReservedExternalDependency};

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
    if data.dependencies[0]
      .as_any()
      .is::<ReservedExternalDependency>()
    {
      let plugin_driver = data.build_context.plugin_driver.clone();
      if let Some(false) = plugin_driver
        .normal_module_factory_hooks
        .before_resolve
        .call(data)
        .await?
      {
        return Ok(ModuleFactoryResult::default());
      }
      let module = plugin_driver
        .normal_module_factory_hooks
        .factorize
        .call(data)
        .await?;
      let Some(mut module) = module.filter(|module| module.as_external_module().is_some()) else {
        return Ok(ModuleFactoryResult::default());
      };
      plugin_driver
        .normal_module_factory_hooks
        .after_factorize
        .call(data, &mut module)
        .await?;
      return Ok(ModuleFactoryResult::new_with_module(module));
    }
    let dep: &LazyCompilationDependency = data.dependencies[0]
      .as_any()
      .downcast_ref()
      .expect("should be lazy compile dependency");
    let options = dep.options();

    data.issuer = options.issuer.clone();
    data.issuer_layer = options.issuer_layer.clone();
    data.resolve_options = options.resolve_options.clone();
    data
      .file_dependencies
      .extend(options.file_dependencies.iter().cloned());
    data
      .context_dependencies
      .extend(options.context_dependencies.iter().cloned());
    data
      .missing_dependencies
      .extend(options.missing_dependencies.iter().cloned());
    data.diagnostics.extend(options.diagnostics.iter().cloned());

    self.normal_module_factory.create(data).await
  }
}
