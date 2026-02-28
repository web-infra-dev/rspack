use async_trait::async_trait;
use rspack_core::{
  Dependency, DependencyType, ModuleExt, ModuleFactory, ModuleFactoryCreateData,
  ModuleFactoryResult,
};
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
    if *dep.dependency_type() == DependencyType::ShareContainerEntry {
      Ok(ModuleFactoryResult::new_with_module(
        ContainerEntryModule::new_share_container_entry(
          dep.name.clone(),
          dep.request.clone().expect("should have request"),
          dep.version.clone().expect("should have version"),
        )
        .boxed(),
      ))
    } else {
      Ok(ModuleFactoryResult::new_with_module(
        ContainerEntryModule::new(
          dep.name.clone(),
          dep.exposes.clone(),
          dep.share_scope.clone(),
          dep.enhanced,
        )
        .boxed(),
      ))
    }
  }
}
