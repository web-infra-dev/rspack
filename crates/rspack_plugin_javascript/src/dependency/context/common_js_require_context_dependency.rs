use rspack_core::{AsModuleDependency, ContextDependency};
use rspack_core::{ContextOptions, Dependency, TemplateReplaceSource};
use rspack_core::{DependencyCategory, DependencyId, DependencyTemplate};
use rspack_core::{DependencyType, ErrorSpan, TemplateContext};

use super::{
  context_dependency_template_as_require_call, create_resource_identifier_for_context_dependency,
};

#[derive(Debug, Clone)]
pub struct CommonJsRequireContextDependency {
  callee_start: u32,
  callee_end: u32,
  args_end: u32,
  id: DependencyId,
  options: ContextOptions,
  span: Option<ErrorSpan>,
  resource_identifier: String,
  optional: bool,
}

impl CommonJsRequireContextDependency {
  pub fn new(
    callee_start: u32,
    callee_end: u32,
    args_end: u32,
    options: ContextOptions,
    span: Option<ErrorSpan>,
    optional: bool,
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
      optional,
    }
  }
}

impl Dependency for CommonJsRequireContextDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CommonJSRequireContext
  }

  fn span(&self) -> Option<ErrorSpan> {
    self.span
  }

  fn dependency_debug_name(&self) -> &'static str {
    "CommonJsRequireContextDependency"
  }
}

impl ContextDependency for CommonJsRequireContextDependency {
  fn request(&self) -> &str {
    &self.options.request
  }

  fn options(&self) -> &ContextOptions {
    &self.options
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

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn type_prefix(&self) -> rspack_core::ContextTypePrefix {
    rspack_core::ContextTypePrefix::Normal
  }
}

impl DependencyTemplate for CommonJsRequireContextDependency {
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

impl AsModuleDependency for CommonJsRequireContextDependency {}
