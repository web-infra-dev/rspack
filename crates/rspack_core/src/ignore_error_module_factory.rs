use std::sync::Arc;

use rspack_error::Result;

use crate::{ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, NormalModuleFactory};

#[derive(Debug)]
pub struct IgnoreErrorModuleFactory {
  pub normal_module_factory: Arc<NormalModuleFactory>,
}

#[async_trait::async_trait]
impl ModuleFactory for IgnoreErrorModuleFactory {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    if let Ok(factory_result) = self.normal_module_factory.create(data).await {
      return Ok(factory_result);
    }
    Ok(Default::default())
  }
}
