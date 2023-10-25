use rspack_core::{
  AsDependencyTemplate, Dependency, DependencyCategory, DependencyId, DependencyType, ErrorSpan,
  ExtendedReferencedExport, ModuleDependency, ModuleGraph, RuntimeSpec,
};
use swc_core::ecma::atoms::JsWord;

use crate::WasmNode;

#[derive(Debug, Clone)]
pub struct WasmImportDependency {
  id: DependencyId,
  name: JsWord,
  request: String,
  // only_direct_import: bool,
  /// the WASM AST node
  pub desc: WasmNode,

  span: Option<ErrorSpan>,
}

impl WasmImportDependency {
  pub fn new(request: String, name: String, desc: WasmNode) -> Self {
    Self {
      id: DependencyId::new(),
      name: name.into(),
      request,
      desc,
      // only_direct_import,
      span: None,
    }
  }
  pub fn name(&self) -> &str {
    &self.name
  }
}

impl Dependency for WasmImportDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Wasm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::WasmImport
  }

  fn span(&self) -> Option<ErrorSpan> {
    self.span
  }

  fn dependency_debug_name(&self) -> &'static str {
    "WasmImportDependency"
  }
}

impl ModuleDependency for WasmImportDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    vec![ExtendedReferencedExport::Array(vec![self.name.clone()])]
  }
}

impl AsDependencyTemplate for WasmImportDependency {}
