use async_trait::async_trait;
use rspack_core::{ModuleExt, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult};
use rspack_error::Result;

use super::{
  container_entry_dependency::ContainerEntryDependency,
  container_entry_module::ContainerEntryModule,
};

#[derive(Debug)]
pub struct ContainerEntryModuleFactory;

#[async_trait]
impl ModuleFactory for ContainerEntryModuleFactory {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    let dep = data.dependencies[0]
      .downcast_ref::<ContainerEntryDependency>()
      .expect("dependency of ContainerEntryModuleFactory should be ContainerEntryDependency");
    Ok(ModuleFactoryResult::new_with_module(
      ContainerEntryModule::new(
        dep.name.as_str(),
        dep.exposes.clone(),
        dep.share_scope.clone(),
        dep.enhanced,
      )
      .boxed(),
    ))
  }
}
