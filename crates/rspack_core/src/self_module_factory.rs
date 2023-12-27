use rspack_error::impl_empty_diagnosable_trait;
use rspack_error::Diagnostic;
use rspack_error::Result;

use crate::SelfModule;
use crate::{ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult};

#[derive(Debug)]
pub struct SelfModuleFactory;

#[async_trait::async_trait]
impl ModuleFactory for SelfModuleFactory {
  async fn create(
    &self,
    data: ModuleFactoryCreateData,
  ) -> Result<(ModuleFactoryResult, Vec<Diagnostic>)> {
    let issur = data
      .issuer_identifier
      .expect("self module must have issuer");
    Ok((
      ModuleFactoryResult::new(Box::new(SelfModule::new(issur))),
      vec![],
    ))
  }
}

impl_empty_diagnosable_trait!(SelfModuleFactory);
