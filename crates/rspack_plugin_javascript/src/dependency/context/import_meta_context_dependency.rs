use std::sync::{Arc, Mutex};

use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
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
pub struct ImportMetaContextDependency {
  id: DependencyId,
  options: ContextOptions,
  range: DependencyRange,
  resource_identifier: ResourceIdentifier,
  optional: bool,
  #[cacheable(with=Skip)]
  critical: Arc<Mutex<Option<Diagnostic>>>,
  #[cacheable(with=rspack_cacheable::with::Skip)]
  factorize_info: std::sync::Arc<std::sync::Mutex<FactorizeInfo>>,
}

impl ImportMetaContextDependency {
  pub fn new(options: ContextOptions, range: DependencyRange, optional: bool) -> Self {
    let resource_identifier = create_resource_identifier_for_context_dependency(None, &options);
    Self {
      options,
      range,
      resource_identifier,
      optional,
      id: DependencyId::new(),
      critical: Default::default(),
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
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
impl DependencyCodeGeneration for ImportMetaContextDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ImportMetaContextDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for ImportMetaContextDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ImportMetaContextDependencyTemplate;

impl ImportMetaContextDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::ImportMetaContext)
  }
}

impl DependencyTemplate for ImportMetaContextDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ImportMetaContextDependency>()
      .expect("ImportMetaContextDependencyTemplate should be used for ImportMetaContextDependency");

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
