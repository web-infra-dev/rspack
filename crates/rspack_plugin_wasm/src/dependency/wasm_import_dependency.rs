use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, Unsupported},
};
use rspack_core::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyRange, DependencyType, ExtendedReferencedExport, FactorizeInfo, ModuleDependency,
  ModuleGraph, RuntimeSpec,
};
use swc_core::ecma::atoms::Atom;

use crate::WasmNode;

#[allow(dead_code)]
#[cacheable]
#[derive(Debug, Clone)]
pub struct WasmImportDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  name: Atom,
  request: String,
  // only_direct_import: bool,
  /// the WASM AST node
  #[cacheable(with=Unsupported)]
  pub desc: WasmNode,
  span: Option<DependencyRange>,
  factorize_info: FactorizeInfo,
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
      factorize_info: Default::default(),
    }
  }
  pub fn name(&self) -> &str {
    &self.name
  }
}

#[cacheable_dyn]
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

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    vec![ExtendedReferencedExport::Array(vec![self.name.clone()])]
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
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

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsDependencyTemplate for WasmImportDependency {}

impl AsContextDependency for WasmImportDependency {}
