use std::sync::Mutex;

use async_trait::async_trait;
use rspack_core::{ModuleDependency, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult};
use rspack_error::{Diagnosable, Diagnostic, Result};

use super::{
  provide_shared_dependency::ProvideSharedDependency, provide_shared_module::ProvideSharedModule,
};

#[derive(Debug, Default)]
pub struct ProvideSharedModuleFactory {
  diagnostics: Mutex<Vec<Diagnostic>>,
}

#[async_trait]
impl ModuleFactory for ProvideSharedModuleFactory {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    let dep = data
      .dependency
      .downcast_ref::<ProvideSharedDependency>()
      .expect("dependency of ProvideSharedModuleFactory should be ProvideSharedDependency");
    Ok(ModuleFactoryResult::new_with_module(Box::new(
      ProvideSharedModule::new(
        dep.share_scope.clone(),
        dep.name.clone(),
        dep.version.clone(),
        dep.request().to_owned(),
        dep.eager,
      ),
    )))
  }
}

impl Diagnosable for ProvideSharedModuleFactory {
  fn add_diagnostic(&self, diagnostic: Diagnostic) {
    self
      .diagnostics
      .lock()
      .expect("should be able to lock diagnostics")
      .push(diagnostic);
  }

  fn add_diagnostics(&self, mut diagnostics: Vec<Diagnostic>) {
    self
      .diagnostics
      .lock()
      .expect("should be able to lock diagnostics")
      .append(&mut diagnostics);
  }

  fn clone_diagnostics(&self) -> Vec<Diagnostic> {
    self
      .diagnostics
      .lock()
      .expect("should be able to lock diagnostics")
      .iter()
      .cloned()
      .collect()
  }
}
