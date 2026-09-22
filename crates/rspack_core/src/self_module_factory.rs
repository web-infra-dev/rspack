use rspack_error::Result;
#[cfg(allocative)]
use rspack_util::allocative;

use crate::{ModuleExt, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, SelfModule};

#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
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
