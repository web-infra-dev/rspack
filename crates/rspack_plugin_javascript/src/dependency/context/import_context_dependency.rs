use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsModuleDependency, ContextDependency, ContextOptions, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, FactorizeInfo, ModuleGraph, ModuleGraphCacheArtifact,
  ResourceIdentifier, TemplateContext, TemplateReplaceSource,
};
use rspack_error::Diagnostic;
use swc_core::atoms::Atom;

use super::{
  context_dependency_template_as_require_call, create_resource_identifier_for_context_dependency,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ImportContextDependency {
  id: DependencyId,
  options: ContextOptions,
  range: DependencyRange,
  value_range: DependencyRange,
  resource_identifier: ResourceIdentifier,
  optional: bool,
  critical: Option<Diagnostic>,
  factorize_info: FactorizeInfo,
}

impl ImportContextDependency {
  pub fn new(
    options: ContextOptions,
    range: DependencyRange,
    value_range: DependencyRange,
    optional: bool,
  ) -> Self {
    let mut resource_identifier =
      create_resource_identifier_for_context_dependency(None, &options).to_string();
    if let Some(attributes) = &options.attributes {
      resource_identifier
        .push_str(&serde_json::to_string(attributes).expect("json stringify failed"));
    }
    Self {
      options,
      range,
      value_range,
      id: DependencyId::new(),
      resource_identifier: resource_identifier.into(),
      optional,
      critical: None,
      factorize_info: Default::default(),
    }
  }

  pub fn set_referenced_exports(&mut self, referenced_exports: Vec<Vec<Atom>>) {
    self.options.referenced_exports = Some(referenced_exports);
  }
}

#[cacheable_dyn]
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

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }

  fn get_diagnostics(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
  ) -> Option<Vec<Diagnostic>> {
    if let Some(critical) = self.critical() {
      return Some(vec![critical.clone()]);
    }
    None
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

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn type_prefix(&self) -> rspack_core::ContextTypePrefix {
    rspack_core::ContextTypePrefix::Import
  }

  fn critical(&self) -> &Option<Diagnostic> {
    &self.critical
  }

  fn critical_mut(&mut self) -> &mut Option<Diagnostic> {
    &mut self.critical
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ImportContextDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ImportContextDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for ImportContextDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ImportContextDependencyTemplate;

impl ImportContextDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::ImportContext)
  }
}

impl DependencyTemplate for ImportContextDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ImportContextDependency>()
      .expect("ImportContextDependencyTemplate should be used for ImportContextDependency");

    context_dependency_template_as_require_call(
      dep,
      source,
      code_generatable_context,
      dep.range,
      Some(&dep.value_range),
    );
  }
}
