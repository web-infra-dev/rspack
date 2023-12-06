use async_trait::async_trait;
use rspack_core::{ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult};
use rspack_error::{internal_error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};

use super::{fallback_dependency::FallbackDependency, fallback_module::FallbackModule};

#[derive(Debug)]
pub struct FallbackModuleFactory;

#[async_trait]
impl ModuleFactory for FallbackModuleFactory {
  async fn create(
    &self,
    data: ModuleFactoryCreateData,
  ) -> Result<TWithDiagnosticArray<ModuleFactoryResult>> {
    let dep = data
      .dependency
      .downcast_ref::<FallbackDependency>()
      .ok_or_else(|| {
        internal_error!("dependency of FallbackModuleFactory should be FallbackDependency")
      })?;
    Ok(
      ModuleFactoryResult::new(Box::new(FallbackModule::new(dep.requests.clone())))
        .with_empty_diagnostic(),
    )
  }
}
