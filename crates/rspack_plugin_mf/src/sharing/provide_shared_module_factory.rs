use std::borrow::Cow;

use async_trait::async_trait;
use rspack_core::{ModuleDependency, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult};
use rspack_error::{Diagnosable, Diagnostic, Result};

use super::{
  provide_shared_dependency::ProvideSharedDependency, provide_shared_module::ProvideSharedModule,
};

#[derive(Debug, Default)]
pub struct ProvideSharedModuleFactory {
  diagnostics: Vec<Diagnostic>,
}

#[async_trait]
impl ModuleFactory for ProvideSharedModuleFactory {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    let dep = data.dependencies[0]
      .downcast_ref::<ProvideSharedDependency>()
      .expect("dependency of ProvideSharedModuleFactory should be ProvideSharedDependency");
    Ok(ModuleFactoryResult::new_with_module(Box::new(
      ProvideSharedModule::new(
        dep.share_scope.clone(),
        dep.name.clone(),
        dep.version.clone(),
        dep.request().to_owned(),
        dep.eager,
        dep.singleton,
        dep.required_version.clone(),
        dep.strict_version,
      ),
    )))
  }
}

impl Diagnosable for ProvideSharedModuleFactory {
  fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
    self.diagnostics.push(diagnostic);
  }

  fn add_diagnostics(&mut self, mut diagnostics: Vec<Diagnostic>) {
    self.diagnostics.append(&mut diagnostics);
  }

  fn diagnostics(&self) -> Cow<[Diagnostic]> {
    Cow::Borrowed(&self.diagnostics)
  }
}
