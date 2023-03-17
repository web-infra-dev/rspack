use crate::{
  CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, Dependency, DependencyCategory,
  DependencyId, DependencyType, ErrorSpan, ModuleDependency, ModuleIdentifier,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StaticExportsDependency {
  pub id: Option<DependencyId>,
  exports: Vec<String>,
  can_mangle: bool,

  request: String,
}
impl StaticExportsDependency {
  pub fn new(exports: Vec<String>, can_mangle: bool) -> Self {
    Self {
      id: None,
      request: "".to_string(),
      exports,
      can_mangle,
    }
  }
}

impl Dependency for StaticExportsDependency {
  fn id(&self) -> Option<&DependencyId> {
    self.id.as_ref()
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    None
  }
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Unknown
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::StaticExports
  }
}

impl ModuleDependency for StaticExportsDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    None
  }
}

impl CodeGeneratable for StaticExportsDependency {
  fn generate(
    &self,
    _code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    todo!()
  }
}
