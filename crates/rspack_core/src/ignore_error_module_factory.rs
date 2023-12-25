use std::sync::Arc;

use rspack_error::{impl_empty_diagnosable_trait, Diagnostic, Result};

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
  ) -> Result<(ModuleFactoryResult, Vec<Diagnostic>)> {
    let factory_result = self.normal_module_factory.create(data).await?;
    Ok(factory_result)
  }
}

impl_empty_diagnosable_trait!(IgnoreErrorModuleFactory);
