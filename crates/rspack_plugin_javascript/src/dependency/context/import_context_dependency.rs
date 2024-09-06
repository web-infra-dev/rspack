use rspack_core::{
  AsModuleDependency, Compilation, ContextDependency, RealDependencyLocation, RuntimeSpec,
};
use rspack_core::{ContextOptions, Dependency, TemplateReplaceSource};
use rspack_core::{DependencyCategory, DependencyId, DependencyTemplate};
use rspack_core::{DependencyType, ErrorSpan, TemplateContext};

use super::{
  context_dependency_template_as_require_call, create_resource_identifier_for_context_dependency,
};

#[derive(Debug, Clone)]
pub struct ImportContextDependency {
  id: DependencyId,
  options: ContextOptions,
  range: RealDependencyLocation,
  range_callee: (u32, u32),
  resource_identifier: String,
  optional: bool,
}

impl ImportContextDependency {
  pub fn new(
    options: ContextOptions,
    range: RealDependencyLocation,
    range_callee: (u32, u32),
    optional: bool,
  ) -> Self {
    let resource_identifier = create_resource_identifier_for_context_dependency(None, &options);
    Self {
      options,
      range,
      range_callee,
      id: DependencyId::new(),
      resource_identifier,
      optional,
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
    Some(ErrorSpan::new(self.range.start, self.range.end))
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
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

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn type_prefix(&self) -> rspack_core::ContextTypePrefix {
    rspack_core::ContextTypePrefix::Import
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
      self.range_callee.0,
      self.range_callee.1,
      self.range.end,
    );
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }

  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }
}

impl AsModuleDependency for ImportContextDependency {}
