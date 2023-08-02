use rspack_core::{
  Dependency, DependencyCategory, DependencyId, DependencyType, ErrorSpan, ModuleDependency,
};
use swc_core::ecma::atoms::JsWord;

use crate::WasmNode;

#[derive(Debug, Clone)]
pub struct WasmImportDependency {
  id: DependencyId,
  name: String,
  request: JsWord,
  // only_direct_import: bool,
  /// the WASM AST node
  pub desc: WasmNode,

  span: Option<ErrorSpan>,
}

impl WasmImportDependency {
  pub fn new(request: JsWord, name: String, desc: WasmNode) -> Self {
    Self {
      id: DependencyId::new(),
      name,
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
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Wasm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::WasmImport
  }
}

impl ModuleDependency for WasmImportDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn request(&self) -> &JsWord {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }

  fn set_request(&mut self, request: JsWord) {
    self.request = request;
  }
}
