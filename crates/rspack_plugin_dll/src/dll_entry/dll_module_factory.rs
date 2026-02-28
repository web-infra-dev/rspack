use async_trait::async_trait;
use rspack_core::{ModuleExt, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult};
use rspack_error::Result;

use super::{dll_entry_dependency::DllEntryDependency, dll_module::DllModule};

#[derive(Debug)]
pub(crate) struct DllModuleFactory;

#[async_trait]
impl ModuleFactory for DllModuleFactory {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    let dll_entry_dependency = data.dependencies[0]
      .as_any()
      .downcast_ref::<DllEntryDependency>()
      .expect("unreachable");

    Ok(ModuleFactoryResult {
      module: Some(DllModule::new(dll_entry_dependency).boxed()),
    })
  }
}
