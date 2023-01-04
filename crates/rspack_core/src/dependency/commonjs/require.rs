use crate::{
  dependency::{
    CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, Dependency, DependencyCategory,
    ModuleDependency,
  },
  DependencyType, ModuleIdentifier,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct CommonJSRequireDependency {
  parent_module_identifier: Option<ModuleIdentifier>,
  request: String,
  user_request: String,
}

impl CommonJSRequireDependency {
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

impl Dependency for CommonJSRequireDependency {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsRequire
  }
}

// impl_module_dependency_cast!(CommonJSRequireDependency);

impl ModuleDependency for CommonJSRequireDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.user_request
  }
}

impl CodeGeneratable for CommonJSRequireDependency {
  fn generate(
    &self,
    _code_generatable_context: CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    todo!()
  }
}
