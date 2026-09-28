use async_trait::async_trait;
use rspack_core::{ModuleExt, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult};
use rspack_error::Result;
#[cfg(allocative)]
use rspack_util::allocative;

use super::{fallback_dependency::FallbackDependency, fallback_module::FallbackModule};

#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct FallbackModuleFactory;

#[async_trait]
impl ModuleFactory for FallbackModuleFactory {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    let dep = data.dependencies[0]
      .downcast_ref::<FallbackDependency>()
      .expect("dependency of FallbackModuleFactory should be FallbackDependency");
    Ok(ModuleFactoryResult::new_with_module(
      FallbackModule::new(
        dep.requests.clone(),
        data.build_context.compiler_options.experiments.runtime_mode,
      )
      .boxed(),
    ))
  }
}
