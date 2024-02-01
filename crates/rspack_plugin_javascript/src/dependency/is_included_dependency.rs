use rspack_core::{
  AsContextDependency, Dependency, DependencyId, DependencyTemplate, DependencyType,
  ModuleDependency, TemplateContext, TemplateReplaceSource,
};

#[derive(Debug, Clone)]
pub struct WebpackIsIncludedDependency {
  pub start: u32,
  pub end: u32,
  pub id: DependencyId,
  pub request: String,
}

impl WebpackIsIncludedDependency {
  pub fn new(start: u32, end: u32, request: String) -> Self {
    Self {
      start,
      end,
      id: DependencyId::default(),
      request,
    }
  }
}

impl AsContextDependency for WebpackIsIncludedDependency {}

impl Dependency for WebpackIsIncludedDependency {
  fn dependency_debug_name(&self) -> &'static str {
    "WebpackIsIncludedDependency"
  }

  fn dependency_type(&self) -> &rspack_core::DependencyType {
    &DependencyType::WebpackIsIncluded
  }

  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }
}

impl ModuleDependency for WebpackIsIncludedDependency {
  fn weak(&self) -> bool {
    true
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &rspack_core::ModuleGraph,
    _runtime: Option<&rspack_core::RuntimeSpec>,
  ) -> Vec<rspack_core::ExtendedReferencedExport> {
    vec![]
  }

  fn request(&self) -> &str {
    &self.request
  }
}

impl DependencyTemplate for WebpackIsIncludedDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext { compilation, .. } = code_generatable_context;

    let included = compilation
      .module_graph
      .connection_by_dependency(&self.id)
      .map(|connection| {
        compilation
          .chunk_graph
          .get_number_of_module_chunks(connection.module_identifier)
          > 0
      })
      .unwrap_or(false);

    source.replace(self.start, self.end, included.to_string().as_str(), None);
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}
