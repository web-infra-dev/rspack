use async_trait::async_trait;
use rspack_core::{ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult};
use rspack_error::{impl_empty_diagnosable_trait, Result};

use super::{fallback_dependency::FallbackDependency, fallback_module::FallbackModule};

#[derive(Debug)]
pub struct FallbackModuleFactory;

#[async_trait]
impl ModuleFactory for FallbackModuleFactory {
  async fn create(&self, data: ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    let dep = data
      .dependency
      .downcast_ref::<FallbackDependency>()
      .expect("dependency of FallbackModuleFactory should be FallbackDependency");
    Ok(ModuleFactoryResult::new_with_module(Box::new(
      FallbackModule::new(dep.requests.clone()),
    )))
  }
}

impl_empty_diagnosable_trait!(FallbackModuleFactory);
