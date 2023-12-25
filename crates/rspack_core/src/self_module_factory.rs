use rspack_error::IntoTWithDiagnosticArray;
use rspack_error::Result;
use rspack_error::TWithDiagnosticArray;

use crate::SelfModule;
use crate::{ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult};

#[derive(Debug)]
pub struct SelfModuleFactory;

#[async_trait::async_trait]
impl ModuleFactory for SelfModuleFactory {
  async fn create(
    &self,
    data: ModuleFactoryCreateData,
  ) -> Result<TWithDiagnosticArray<ModuleFactoryResult>> {
    let issur = data
      .issuer_identifier
      .expect("self module must have issuer");
    Ok(ModuleFactoryResult::new(Box::new(SelfModule::new(issur))).with_empty_diagnostic())
  }
}
