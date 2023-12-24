use std::sync::Arc;

use rspack_error::IntoTWithDiagnosticArray;
use rspack_error::Result;
use rspack_error::TWithDiagnosticArray;

use crate::{ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, NormalModuleFactory};

#[derive(Debug)]
pub struct IgnoreErrorModuleFactory {
  pub normal_module_factory: Arc<NormalModuleFactory>,
}

#[async_trait::async_trait]
impl ModuleFactory for IgnoreErrorModuleFactory {
  async fn create(
    &self,
    data: ModuleFactoryCreateData,
  ) -> Result<TWithDiagnosticArray<ModuleFactoryResult>> {
    let (factory_result, _) = self
      .normal_module_factory
      .create(data)
      .await?
      .split_into_parts();
    Ok(factory_result.with_diagnostic(vec![]))
  }
}
