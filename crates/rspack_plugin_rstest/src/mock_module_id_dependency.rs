use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId,
  DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType, ExportsInfoArtifact,
  ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact, ReferencedExport, RuntimeSpec,
  TemplateContext, TemplateReplaceSource,
};

use crate::import_dependency::module_id_rstest;

#[cacheable]
#[derive(Debug)]
pub struct MockModuleIdDependency {
  pub id: DependencyId,
  pub request: String,
  pub weak: bool,
  range: DependencyRange,
  optional: bool,
  category: DependencyCategory,
  pub suffix: Option<String>,
  missing_module_fallback: Option<String>,
}

#[allow(clippy::too_many_arguments)]
impl MockModuleIdDependency {
  pub fn new(
    request: String,
    range: DependencyRange,
    weak: bool,
    optional: bool,
    category: DependencyCategory,
    suffix: Option<String>,
  ) -> Self {
    Self {
      range,
      request,
      weak,
      optional,
      id: DependencyId::new(),
      category,
      suffix,
      missing_module_fallback: None,
    }
  }

  pub fn with_request(&self, request: String) -> Self {
    Self {
      id: self.id,
      request,
      weak: self.weak,
      range: self.range,
      optional: self.optional,
      category: self.category,
      suffix: self.suffix.clone(),
      missing_module_fallback: self.missing_module_fallback.clone(),
    }
  }

  pub fn with_missing_module_fallback(mut self, fallback: String) -> Self {
    self.missing_module_fallback = Some(fallback);
    self
  }

  pub fn has_missing_module_fallback(&self) -> bool {
    self.missing_module_fallback.is_some()
  }
}

#[cacheable_dyn]
impl Dependency for MockModuleIdDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &self.category
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RstestMockModuleId
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ReferencedExport> {
    vec![]
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for MockModuleIdDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn weak(&self) -> bool {
    self.weak
  }

  fn get_optional(&self) -> bool {
    self.optional
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for MockModuleIdDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(MockModuleIdDependencyTemplate::template_type())
  }
}

impl AsContextDependency for MockModuleIdDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct MockModuleIdDependencyTemplate;

impl MockModuleIdDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::RstestMockModuleId)
  }
}

impl DependencyTemplate for MockModuleIdDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<MockModuleIdDependency>()
      .expect("MockModuleIdDependencyTemplate should only be used for MockModuleIdDependency");

    let module_id = if code_generatable_context
      .compilation
      .get_module_graph()
      .module_identifier_by_dependency_id(&dep.id)
      .is_none()
      && let Some(fallback) = &dep.missing_module_fallback
    {
      fallback.clone()
    } else {
      module_id_rstest(
        code_generatable_context.compilation,
        code_generatable_context.runtime_template,
        &dep.id,
        &dep.request,
        dep.weak,
      )
    };

    source.replace(
      dep.range.start,
      dep.range.end,
      format!("{}{}", module_id, dep.suffix.as_deref().unwrap_or("")),
      None,
    );
  }
}
