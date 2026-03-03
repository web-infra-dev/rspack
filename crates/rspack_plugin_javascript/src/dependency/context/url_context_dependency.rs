use std::sync::{Arc, Mutex};

use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  AsModuleDependency, ContextDependency, ContextOptions, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExportsInfoArtifact, FactorizeInfo, ModuleGraph,
  ModuleGraphCacheArtifact, ResourceIdentifier, TemplateContext, TemplateReplaceSource,
};
use rspack_error::Diagnostic;

use super::{
  context_dependency_template_as_require_call, create_resource_identifier_for_context_dependency,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct URLContextDependency {
  id: DependencyId,
  range: DependencyRange,
  value_range: DependencyRange,
  resource_identifier: ResourceIdentifier,
  options: ContextOptions,
  optional: bool,
  #[cacheable(with=Skip)]
  critical: Arc<Mutex<Option<Diagnostic>>>,
  #[cacheable(with=rspack_cacheable::with::As<FactorizeInfo>)]
  factorize_info: std::sync::Arc<std::sync::Mutex<FactorizeInfo>>,
}

impl URLContextDependency {
  pub fn new(
    options: ContextOptions,
    range: DependencyRange,
    value_range: DependencyRange,
    optional: bool,
  ) -> Self {
    let resource_identifier = create_resource_identifier_for_context_dependency(None, &options);
    Self {
      range,
      value_range,
      options,
      resource_identifier,
      optional,
      id: DependencyId::new(),
      critical: Default::default(),
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for URLContextDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Url
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::NewUrlContext
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
      return Some(vec![critical]);
    }
    None
  }
}

impl ContextDependency for URLContextDependency {
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

  fn critical(&self) -> Option<Diagnostic> {
    self
      .critical
      .lock()
      .expect("context dependency critical poisoned")
      .clone()
  }

  fn set_critical(&self, diagnostic: Option<Diagnostic>) {
    *self
      .critical
      .lock()
      .expect("context dependency critical poisoned") = diagnostic;
  }

  fn factorize_info(&self) -> std::sync::MutexGuard<'_, FactorizeInfo> {
    self
      .factorize_info
      .lock()
      .expect("dependency factorize_info poisoned")
  }

  fn set_factorize_info(&self, info: FactorizeInfo) {
    *self
      .factorize_info
      .lock()
      .expect("dependency factorize_info poisoned") = info;
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for URLContextDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(URLContextDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for URLContextDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct URLContextDependencyTemplate;

impl URLContextDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::NewUrlContext)
  }
}

impl DependencyTemplate for URLContextDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<URLContextDependency>()
      .expect("URLContextDependencyTemplate should be used for URLContextDependency");

    context_dependency_template_as_require_call(
      dep,
      source,
      code_generatable_context,
      &dep.range,
      Some(&dep.value_range),
    );
  }
}
