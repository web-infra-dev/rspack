use core::hash::Hash;
use std::hash::Hasher;

use rspack_core::{
  CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, Dependency, DependencyCategory,
  DependencyId, DependencyType, ErrorSpan, ModuleDependency, ModuleIdentifier,
};

use crate::WasmNode;

#[derive(Debug, Clone)]
pub struct WasmImportDependency {
  id: Option<DependencyId>,
  parent_module_identifier: Option<ModuleIdentifier>,
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
      parent_module_identifier: None,
      name,
      request,
      desc,
      // only_direct_import,
      span: None,
    }
  }
}

impl Dependency for WasmImportDependency {
  fn id(&self) -> Option<&DependencyId> {
    self.id.as_ref()
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn set_parent_module_identifier(&mut self, identifier: Option<ModuleIdentifier>) {
    self.parent_module_identifier = identifier;
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
}

impl CodeGeneratable for WasmImportDependency {
  fn generate(
    &self,
    _code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    todo!()
  }
}

impl PartialEq for WasmImportDependency {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
      && self.parent_module_identifier == other.parent_module_identifier
      && self.name == other.name
      && self.request == other.request
  }
}
impl Eq for WasmImportDependency {}
impl Hash for WasmImportDependency {
  fn hash<H: Hasher>(&self, state: &mut H) {}
}
