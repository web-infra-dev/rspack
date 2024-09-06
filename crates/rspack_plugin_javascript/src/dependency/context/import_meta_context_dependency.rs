use rspack_core::{
  module_raw, AsModuleDependency, Compilation, ContextDependency, RealDependencyLocation,
  RuntimeSpec,
};
use rspack_core::{ContextOptions, Dependency, DependencyCategory, DependencyId};
use rspack_core::{DependencyTemplate, DependencyType, ErrorSpan};
use rspack_core::{TemplateContext, TemplateReplaceSource};

use super::create_resource_identifier_for_context_dependency;

#[derive(Debug, Clone)]
pub struct ImportMetaContextDependency {
  id: DependencyId,
  options: ContextOptions,
  range: RealDependencyLocation,
  resource_identifier: String,
  optional: bool,
}

impl ImportMetaContextDependency {
  pub fn new(options: ContextOptions, range: RealDependencyLocation, optional: bool) -> Self {
    let resource_identifier = create_resource_identifier_for_context_dependency(None, &options);
    Self {
      options,
      range,
      resource_identifier,
      optional,
      id: DependencyId::new(),
    }
  }
}

impl Dependency for ImportMetaContextDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ImportMetaContext
  }

  fn span(&self) -> Option<ErrorSpan> {
    Some(ErrorSpan::new(self.range.start, self.range.end))
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

impl ContextDependency for ImportMetaContextDependency {
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

impl DependencyTemplate for ImportMetaContextDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      compilation,
      runtime_requirements,
      ..
    } = code_generatable_context;

    let content = module_raw(
      compilation,
      runtime_requirements,
      &self.id,
      &self.options.request,
      self.optional,
    );
    source.replace(self.range.start, self.range.end, &content, None);
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

impl AsModuleDependency for ImportMetaContextDependency {}
