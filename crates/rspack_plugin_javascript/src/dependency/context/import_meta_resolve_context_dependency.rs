use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AffectType, AsModuleDependency, ContextDependency, ContextOptions, ContextTypePrefix, Dependency,
  DependencyCategory, DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExportsInfoArtifact, FactorizeInfo, ModuleGraph,
  ModuleGraphCacheArtifact, ResourceIdentifier, TemplateContext, TemplateReplaceSource,
};
use rspack_error::Diagnostic;

use super::{context_dependency_template_as_id, create_resource_identifier_for_context_dependency};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ImportMetaResolveContextDependency {
  id: DependencyId,
  options: ContextOptions,
  range: DependencyRange,
  resource_identifier: ResourceIdentifier,
  optional: bool,
  critical: Option<Diagnostic>,
  factorize_info: FactorizeInfo,
}

impl ImportMetaResolveContextDependency {
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
impl Dependency for ImportMetaResolveContextDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ImportMetaResolveContext
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::True
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

impl ContextDependency for ImportMetaResolveContextDependency {
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
impl DependencyCodeGeneration for ImportMetaResolveContextDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ImportMetaResolveContextDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for ImportMetaResolveContextDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ImportMetaResolveContextDependencyTemplate;

impl ImportMetaResolveContextDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::ImportMetaResolveContext)
  }
}

impl DependencyTemplate for ImportMetaResolveContextDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ImportMetaResolveContextDependency>()
      .expect("ImportMetaResolveContextDependencyTemplate should be used for ImportMetaResolveContextDependency");

    context_dependency_template_as_id(dep, source, code_generatable_context, &dep.range);
  }
}
