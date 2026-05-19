use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyRange, DependencyType, ExportsInfoArtifact, ExtendedReferencedExport, FactorizeInfo,
  ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact, RuntimeSpec,
};
use swc_core::ecma::atoms::Atom;

#[allow(dead_code)]
#[cacheable]
#[derive(Debug, Clone)]
pub struct WasmImportDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  name: Atom,
  request: String,
  // only_direct_import: bool,
  span: Option<DependencyRange>,
  factorize_info: FactorizeInfo,
}

impl WasmImportDependency {
  pub fn new(request: String, name: String) -> Self {
    Self {
      id: DependencyId::new(),
      name: name.into(),
      request,
      // only_direct_import,
      span: None,
      factorize_info: Default::default(),
    }
  }

  pub fn name(&self) -> &Atom {
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
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
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

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsDependencyCodeGeneration for WasmImportDependency {}

impl AsContextDependency for WasmImportDependency {}
