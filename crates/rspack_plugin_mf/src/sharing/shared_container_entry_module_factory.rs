use async_trait::async_trait;
use rspack_core::{ModuleExt, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult};
use rspack_error::Result;

use super::{
  shared_container_entry_dependency::SharedContainerEntryDependency,
  shared_container_entry_module::SharedContainerEntryModule,
};

#[derive(Debug)]
pub struct SharedContainerEntryModuleFactory;

#[async_trait]
impl ModuleFactory for SharedContainerEntryModuleFactory {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    let dep = data.dependencies[0]
      .downcast_ref::<SharedContainerEntryDependency>()
      .expect(
        "dependency of SharedContainerEntryModuleFactory should be SharedContainerEntryDependency",
      );
    Ok(ModuleFactoryResult::new_with_module(
      SharedContainerEntryModule::new(dep.name.clone(), dep.request.clone(), dep.version.clone())
        .boxed(),
    ))
  }
}
