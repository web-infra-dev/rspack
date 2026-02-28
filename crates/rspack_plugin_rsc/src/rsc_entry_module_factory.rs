use async_trait::async_trait;
use rspack_core::{ModuleExt, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult};
use rspack_error::Result;

use crate::{rsc_entry_dependency::RscEntryDependency, rsc_entry_module::RscEntryModule};

#[derive(Debug)]
pub struct RscEntryModuleFactory;

#[async_trait]
impl ModuleFactory for RscEntryModuleFactory {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    let dependency = data.dependencies[0]
      .downcast_ref::<RscEntryDependency>()
      .expect("dependency of RscEntryModuleFactory should be RscEntryDependency");
    Ok(ModuleFactoryResult::new_with_module(
      RscEntryModule::new(dependency.name.clone(), dependency.client_modules.clone()).boxed(),
    ))
  }
}
