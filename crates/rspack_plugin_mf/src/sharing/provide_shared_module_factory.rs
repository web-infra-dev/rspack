use async_trait::async_trait;
use rspack_core::{ModuleDependency, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult};
use rspack_error::{internal_error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};

use super::{
  provide_shared_dependency::ProvideSharedDependency, provide_shared_module::ProvideSharedModule,
};

#[derive(Debug)]
pub struct ProvideSharedModuleFactory;

#[async_trait]
impl ModuleFactory for ProvideSharedModuleFactory {
  async fn create(
    &self,
    data: ModuleFactoryCreateData,
  ) -> Result<TWithDiagnosticArray<ModuleFactoryResult>> {
    let dep = data
      .dependency
      .downcast_ref::<ProvideSharedDependency>()
      .ok_or_else(|| {
        internal_error!(
          "dependency of ProvideSharedModuleFactory should be ProvideSharedDependency"
        )
      })?;
    Ok(
      ModuleFactoryResult::new(Box::new(ProvideSharedModule::new(
        dep.share_scope.clone(),
        dep.name.clone(),
        dep.version.clone(),
        dep.request().to_owned(),
        dep.eager,
      )))
      .with_empty_diagnostic(),
    )
  }
}
