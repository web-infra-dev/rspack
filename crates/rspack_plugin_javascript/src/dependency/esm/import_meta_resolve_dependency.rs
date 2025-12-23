use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId,
  DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType,
  ExtendedReferencedExport, FactorizeInfo, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact,
  RuntimeSpec, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ImportMetaResolveDependency {
  pub id: DependencyId,
  pub request: String,
  range: DependencyRange,
  optional: bool,
  factorize_info: FactorizeInfo,
}

impl ImportMetaResolveDependency {
  pub fn new(request: String, range: DependencyRange, optional: bool) -> Self {
    Self {
      range,
      request,
      optional,
      id: DependencyId::new(),
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for ImportMetaResolveDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ImportMetaResolve
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
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
impl ModuleDependency for ImportMetaResolveDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
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
impl DependencyCodeGeneration for ImportMetaResolveDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ImportMetaResolveDependencyTemplate::template_type())
  }
}

impl AsContextDependency for ImportMetaResolveDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ImportMetaResolveDependencyTemplate;

impl ImportMetaResolveDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::ImportMetaResolve)
  }
}

impl DependencyTemplate for ImportMetaResolveDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ImportMetaResolveDependency>()
      .expect(
        "ImportMetaResolveDependencyTemplate should only be used for ImportMetaResolveDependency",
      );

    source.replace(
      dep.range.start,
      dep.range.end,
      code_generatable_context
        .compilation
        .runtime_template
        .module_id(
          code_generatable_context.compilation,
          &dep.id,
          &dep.request,
          false,
        )
        .as_str(),
      None,
    );
  }
}
