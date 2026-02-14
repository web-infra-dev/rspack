use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId,
  DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType, ExportsInfoArtifact,
  ExtendedReferencedExport, FactorizeInfo, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact,
  RuntimeSpec, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct RequireResolveDependency {
  pub id: DependencyId,
  pub request: String,
  pub weak: bool,
  range: DependencyRange,
  optional: bool,
  factorize_info: FactorizeInfo,
}

impl RequireResolveDependency {
  pub fn new(request: String, range: DependencyRange, weak: bool, optional: bool) -> Self {
    Self {
      range,
      request,
      weak,
      optional,
      id: DependencyId::new(),
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for RequireResolveDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RequireResolve
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
  ) -> Vec<ExtendedReferencedExport> {
    vec![]
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for RequireResolveDependency {
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

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for RequireResolveDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(RequireResolveDependencyTemplate::template_type())
  }
}

impl AsContextDependency for RequireResolveDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct RequireResolveDependencyTemplate;

impl RequireResolveDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::RequireResolve)
  }
}

impl DependencyTemplate for RequireResolveDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<RequireResolveDependency>()
      .expect("RequireResolveDependencyTemplate should only be used for RequireResolveDependency");

    source.replace(
      dep.range.start,
      dep.range.end,
      code_generatable_context
        .runtime_template
        .module_id(
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
