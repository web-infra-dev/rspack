use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, Dependency, DependencyCodeGeneration, DependencyId, DependencyRange,
  DependencyTemplate, DependencyTemplateType, DependencyType, ExtendedReferencedExport,
  FactorizeInfo, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact, RuntimeSpec,
  TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct WebpackIsIncludedDependency {
  pub range: DependencyRange,
  pub id: DependencyId,
  pub request: String,
  factorize_info: FactorizeInfo,
}

impl WebpackIsIncludedDependency {
  pub fn new(range: DependencyRange, request: String) -> Self {
    Self {
      range,
      id: DependencyId::default(),
      request,
      factorize_info: Default::default(),
    }
  }
}

impl AsContextDependency for WebpackIsIncludedDependency {}

#[cacheable_dyn]
impl Dependency for WebpackIsIncludedDependency {
  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::WebpackIsIncluded
  }

  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
  }

  fn id(&self) -> &DependencyId {
    &self.id
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
impl ModuleDependency for WebpackIsIncludedDependency {
  fn weak(&self) -> bool {
    true
  }

  fn request(&self) -> &str {
    &self.request
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for WebpackIsIncludedDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(WebpackIsIncludedDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct WebpackIsIncludedDependencyTemplate;

impl WebpackIsIncludedDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::WebpackIsIncluded)
  }
}

impl DependencyTemplate for WebpackIsIncludedDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<WebpackIsIncludedDependency>()
      .expect("WebpackIsIncludedDependencyTemplate should be used for WebpackIsIncludedDependency");
    let TemplateContext { compilation, .. } = code_generatable_context;

    let included = compilation
      .get_module_graph()
      .connection_by_dependency_id(&dep.id)
      .map(|connection| {
        compilation
          .chunk_graph
          .get_number_of_module_chunks(*connection.module_identifier())
          > 0
      })
      .unwrap_or(false);

    source.replace(
      dep.range.start,
      dep.range.end,
      included.to_string().as_str(),
      None,
    );
  }
}
