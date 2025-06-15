use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  module_id, AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration,
  DependencyId, DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType,
  ExtendedReferencedExport, FactorizeInfo, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact,
  RuntimeSpec, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct MockModuleIdDependency {
  pub id: DependencyId,
  pub request: String,
  pub weak: bool,
  range: DependencyRange,
  optional: bool,
  factorize_info: FactorizeInfo,
  category: DependencyCategory,
}

impl MockModuleIdDependency {
  pub fn new(
    request: String,
    range: DependencyRange,
    weak: bool,
    optional: bool,
    category: DependencyCategory,
  ) -> Self {
    Self {
      range,
      request,
      weak,
      optional,
      id: DependencyId::new(),
      factorize_info: Default::default(),
      category,
    }
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

  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
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

  fn set_request(&mut self, request: String) {
    self.request = request;
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
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

    source.replace(
      dep.range.start,
      dep.range.end,
      module_id(
        code_generatable_context.compilation,
        &dep.id,
        &dep.request,
        dep.weak,
      )
      .as_str(),
      None,
    );
  }
}
