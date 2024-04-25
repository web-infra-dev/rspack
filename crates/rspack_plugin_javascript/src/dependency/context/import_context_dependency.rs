use rspack_core::{AsModuleDependency, ContextDependency};
use rspack_core::{ContextOptions, Dependency, TemplateReplaceSource};
use rspack_core::{DependencyCategory, DependencyId, DependencyTemplate};
use rspack_core::{DependencyType, ErrorSpan, TemplateContext};

use super::{
  context_dependency_template_as_require_call, create_resource_identifier_for_context_dependency,
};

#[derive(Debug, Clone)]
pub struct ImportContextDependency {
  callee_start: u32,
  callee_end: u32,
  args_end: u32,
  id: DependencyId,
  options: ContextOptions,
  span: Option<ErrorSpan>,
  resource_identifier: String,
}

impl ImportContextDependency {
  pub fn new(
    callee_start: u32,
    callee_end: u32,
    args_end: u32,
    options: ContextOptions,
    span: Option<ErrorSpan>,
  ) -> Self {
    let resource_identifier = create_resource_identifier_for_context_dependency(None, &options);
    Self {
      callee_start,
      callee_end,
      args_end,
      options,
      span,
      id: DependencyId::new(),
      resource_identifier,
    }
  }
}

impl Dependency for ImportContextDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ImportContext
  }

  fn span(&self) -> Option<ErrorSpan> {
    self.span
  }

  fn dependency_debug_name(&self) -> &'static str {
    "ImportContextDependency"
  }
}

impl ContextDependency for ImportContextDependency {
  fn options(&self) -> &ContextOptions {
    &self.options
  }

  fn request(&self) -> &str {
    &self.options.request
  }

  fn get_context(&self) -> Option<&str> {
    None
  }

  fn resource_identifier(&self) -> &str {
    &self.resource_identifier
  }

  fn set_request(&mut self, request: String) {
    self.options.request = request;
  }
}

impl DependencyTemplate for ImportContextDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    context_dependency_template_as_require_call(
      self,
      source,
      code_generatable_context,
      self.callee_start,
      self.callee_end,
      self.args_end,
    );
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

impl AsModuleDependency for ImportContextDependency {}
