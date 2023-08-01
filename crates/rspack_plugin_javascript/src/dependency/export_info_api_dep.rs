use rspack_core::{
  ChunkGroupOptions, Dependency, DependencyCategory, DependencyId, DependencyTemplate,
  DependencyType, ErrorSpan, ExportsReferencedType, ModuleDependency, ModuleGraph, RuntimeGlobals,
  RuntimeSpec, TemplateContext, TemplateReplaceSource,
};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct ExportInfoApiDependency {
  start: u32,
  end: u32,
  id: DependencyId,
  export_name: Vec<JsWord>,
  property: JsWord,
  // TODO: runtime_requirements
}

impl ExportInfoApiDependency {
  pub fn new(start: u32, end: u32, export_name: Vec<JsWord>, property: JsWord) -> Self {
    Self {
      start,
      end,
      id: DependencyId::new(),
      export_name,
      property,
    }
  }
}

impl Dependency for ExportInfoApiDependency {
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Unknown
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ExportInfoApi
  }
}

impl ModuleDependency for ExportInfoApiDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn request(&self) -> &str {
    ""
  }

  fn user_request(&self) -> &str {
    ""
  }

  fn span(&self) -> Option<&ErrorSpan> {
    None
  }

  fn as_code_generatable_dependency(&self) -> Option<&dyn DependencyTemplate> {
    Some(self)
  }

  fn set_request(&mut self, request: String) {}

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: &RuntimeSpec,
  ) -> ExportsReferencedType {
    ExportsReferencedType::No
  }
}

impl DependencyTemplate for ExportInfoApiDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext { compilation, .. } = code_generatable_context;
  }
}
