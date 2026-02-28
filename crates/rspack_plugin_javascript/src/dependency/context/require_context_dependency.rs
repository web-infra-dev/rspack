use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsModuleDependency, ContextDependency, ContextOptions, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExportsInfoArtifact, FactorizeInfo, ModuleGraph,
  ModuleGraphCacheArtifact, ResourceIdentifier, TemplateContext, TemplateReplaceSource,
};
use rspack_error::Diagnostic;

use super::create_resource_identifier_for_context_dependency;

#[cacheable]
#[derive(Debug, Clone)]
pub struct RequireContextDependency {
  id: DependencyId,
  options: ContextOptions,
  range: DependencyRange,
  resource_identifier: ResourceIdentifier,
  optional: bool,
  critical: Option<Diagnostic>,
  factorize_info: FactorizeInfo,
}

impl RequireContextDependency {
  pub fn new(options: ContextOptions, range: DependencyRange, optional: bool) -> Self {
    let resource_identifier = create_resource_identifier_for_context_dependency(None, &options);
    Self {
      options,
      range,
      id: DependencyId::new(),
      resource_identifier,
      optional,
      critical: None,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for RequireContextDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RequireContext
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
    _exports_info_artifact: &ExportsInfoArtifact,
  ) -> Option<Vec<Diagnostic>> {
    if let Some(critical) = self.critical() {
      return Some(vec![critical.clone()]);
    }
    None
  }
}

impl ContextDependency for RequireContextDependency {
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

  fn type_prefix(&self) -> rspack_core::ContextTypePrefix {
    rspack_core::ContextTypePrefix::Normal
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
impl DependencyCodeGeneration for RequireContextDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(RequireContextDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for RequireContextDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct RequireContextDependencyTemplate;

impl RequireContextDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::RequireContext)
  }
}

impl DependencyTemplate for RequireContextDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<RequireContextDependency>()
      .expect("RequireContextDependencyTemplate should be used for RequireContextDependency");

    let TemplateContext {
      compilation,
      runtime_template,
      ..
    } = code_generatable_context;

    let content =
      runtime_template.module_raw(compilation, &dep.id, &dep.options.request, dep.optional);
    source.replace(dep.range.start, dep.range.end, &content, None);
  }
}
