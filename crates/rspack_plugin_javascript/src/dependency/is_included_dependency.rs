use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, Compilation, Dependency, DependencyId, DependencyTemplate, DependencyType,
  ExtendedReferencedExport, FactorizeInfo, ModuleDependency, ModuleGraph, RuntimeSpec,
  TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct WebpackIsIncludedDependency {
  pub start: u32,
  pub end: u32,
  pub id: DependencyId,
  pub request: String,
  factorize_info: FactorizeInfo,
}

impl WebpackIsIncludedDependency {
  pub fn new(start: u32, end: u32, request: String) -> Self {
    Self {
      start,
      end,
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

  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
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
impl DependencyTemplate for WebpackIsIncludedDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext { compilation, .. } = code_generatable_context;

    let included = compilation
      .get_module_graph()
      .connection_by_dependency_id(&self.id)
      .map(|connection| {
        compilation
          .chunk_graph
          .get_number_of_module_chunks(*connection.module_identifier())
          > 0
      })
      .unwrap_or(false);

    source.replace(self.start, self.end, included.to_string().as_str(), None);
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }

  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }
}
