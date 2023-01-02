use crate::{
  dependency::{Dependency, ModuleDependency, ModuleDependencyCategory},
  AsAny, ModuleIdentifier,
};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct CommonJsRequireDependency {
  parent_module_identifier: Option<ModuleIdentifier>,
  request: String,
  user_request: String,
}

impl CommonJsRequireDependency {
  pub fn new(
    parent_module_identifier: Option<ModuleIdentifier>,
    request: String,
    user_request: String,
  ) -> Self {
    Self {
      parent_module_identifier,
      request,
      user_request,
    }
  }
}

impl Dependency for CommonJsRequireDependency {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn as_module_dependency(&self) -> Option<&dyn ModuleDependency> {
    Some(self)
  }
}

impl ModuleDependency for CommonJsRequireDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.user_request
  }

  fn category(&self) -> ModuleDependencyCategory {
    ModuleDependencyCategory::CommonJS
  }
}
