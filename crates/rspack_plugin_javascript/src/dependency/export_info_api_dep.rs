use rspack_core::{
  ChunkGroupOptions, Dependency, DependencyCategory, DependencyId, DependencyTemplate,
  DependencyType, ErrorSpan, ExportsReferencedType, ModuleDependency, ModuleGraph, RuntimeGlobals,
  RuntimeSpec, TemplateContext, TemplateReplaceSource, UsageState,
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

impl DependencyTemplate for ExportInfoApiDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext { compilation, .. } = code_generatable_context;
  }
}

// impl ExportInfoApiDependency {
//   fn get_property(&self, export_name: Vec<JsWord>, prop: JsWord) -> Option<UsageState> {}
// }
