use async_trait::async_trait;
use rspack_core::{ModuleExt, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult};
use rspack_error::Result;

use super::{
  share_container_entry_dependency::ShareContainerEntryDependency,
  share_container_entry_module::ShareContainerEntryModule,
};

#[derive(Debug)]
pub struct ShareContainerEntryModuleFactory;

#[async_trait]
impl ModuleFactory for ShareContainerEntryModuleFactory {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    let dep = data.dependencies[0]
      .downcast_ref::<ShareContainerEntryDependency>()
      .expect(
        "dependency of ShareContainerEntryModuleFactory should be ShareContainerEntryDependency",
      );
    Ok(ModuleFactoryResult::new_with_module(
      ShareContainerEntryModule::new(dep.name.clone(), dep.request.clone(), dep.version.clone())
        .boxed(),
    ))
  }
}
