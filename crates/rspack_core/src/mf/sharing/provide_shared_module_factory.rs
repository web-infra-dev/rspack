// TODO: move to rspack_plugin_mf

use async_trait::async_trait;
use rspack_error::{internal_error, IntoTWithRspackDiagnosticArray, Result, TWithDiagnosticArray};

use super::{
  provide_shared_dependency::ProvideSharedDependency, provide_shared_module::ProvideSharedModule,
};
use crate::{ModuleDependency, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult};

pub struct ProvideSharedModuleFactory;

#[async_trait]
impl ModuleFactory for ProvideSharedModuleFactory {
  async fn create(
    mut self,
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
