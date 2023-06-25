use rspack_core::{
  Dependency, DependencyCategory, DependencyId, DependencyType, ErrorSpan, ModuleDependency,
};

use crate::WasmNode;

#[derive(Debug, Clone)]
pub struct WasmImportDependency {
  id: Option<DependencyId>,
  name: String,
  request: String,
  // only_direct_import: bool,
  /// the WASM AST node
  pub desc: WasmNode,

  span: Option<ErrorSpan>,
}

impl WasmImportDependency {
  pub fn new(request: String, name: String, desc: WasmNode) -> Self {
    Self {
      id: None,
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
  fn id(&self) -> Option<DependencyId> {
    self.id
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Wasm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::WasmImport
  }
}

impl ModuleDependency for WasmImportDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }
}
