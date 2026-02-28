use rspack_error::Result;

use crate::{ModuleExt, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, SelfModule};

#[derive(Debug)]
pub struct SelfModuleFactory;

#[async_trait::async_trait]
impl ModuleFactory for SelfModuleFactory {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    let issuer = data
      .issuer_identifier
      .expect("self module must have issuer");
    Ok(ModuleFactoryResult::new_with_module(
      SelfModule::new(issuer).boxed(),
    ))
  }
}
