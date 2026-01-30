use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AffectType, AsModuleDependency, ContextDependency, ContextOptions, ContextTypePrefix, Dependency,
  DependencyCategory, DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, FactorizeInfo, ResourceIdentifier, TemplateContext,
  TemplateReplaceSource,
};
use rspack_error::Diagnostic;

use super::{context_dependency_template_as_id, create_resource_identifier_for_context_dependency};

#[cacheable]
#[derive(Debug, Clone)]
pub struct RequireResolveContextDependency {
  id: DependencyId,
  options: ContextOptions,
  range: DependencyRange,
  resource_identifier: ResourceIdentifier,
  optional: bool,
  critical: Option<Diagnostic>,
  factorize_info: FactorizeInfo,
}

impl RequireResolveContextDependency {
  pub fn new(options: ContextOptions, range: DependencyRange, optional: bool) -> Self {
    let resource_identifier = create_resource_identifier_for_context_dependency(None, &options);
    Self {
      id: DependencyId::new(),
      options,
      range,
      resource_identifier,
      optional,
      critical: None,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for RequireResolveContextDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RequireResolveContext
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::True
  }
}

impl ContextDependency for RequireResolveContextDependency {
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

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn type_prefix(&self) -> ContextTypePrefix {
    ContextTypePrefix::Normal
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
impl DependencyCodeGeneration for RequireResolveContextDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(RequireResolveContextDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for RequireResolveContextDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct RequireResolveContextDependencyTemplate;

impl RequireResolveContextDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::RequireResolveContext)
  }
}

impl DependencyTemplate for RequireResolveContextDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<RequireResolveContextDependency>()
      .expect("RequireResolveContextDependencyTemplate should be used for RequireResolveContextDependency");

    context_dependency_template_as_id(dep, source, code_generatable_context, dep.range);
  }
}
