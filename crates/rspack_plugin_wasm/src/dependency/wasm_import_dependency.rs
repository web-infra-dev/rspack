use rspack_core::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyType, ErrorSpan, ExtendedReferencedExport, ModuleDependency, ModuleGraph, RuntimeSpec,
};
use swc_core::ecma::atoms::Atom;

use crate::WasmNode;

#[derive(Debug, Clone)]
pub struct WasmImportDependency {
  id: DependencyId,
  name: Atom,
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

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    vec![ExtendedReferencedExport::Array(vec![self.name.clone()])]
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
}

impl AsDependencyTemplate for WasmImportDependency {}

impl AsContextDependency for WasmImportDependency {}
