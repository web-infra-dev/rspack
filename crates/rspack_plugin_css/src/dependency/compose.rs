use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyRange, DependencyType, ExportsInfoArtifact, ExtendedReferencedExport, FactorizeInfo,
  ModuleDependency, RuntimeSpec,
};
use rspack_util::atom::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct CssComposeDependency {
  id: DependencyId,
  request: String,
  #[cacheable(with=AsVec<AsPreset>)]
  names: Vec<Atom>,
  range: DependencyRange,
  #[cacheable(with=rspack_cacheable::with::As<FactorizeInfo>)]
  factorize_info: std::sync::Arc<std::sync::Mutex<FactorizeInfo>>,
}

impl CssComposeDependency {
  pub fn new(request: String, names: Vec<Atom>, range: DependencyRange) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      names,
      range,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for CssComposeDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssCompose
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssCompose
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &rspack_core::ModuleGraph,
    _module_graph_cache: &rspack_core::ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    self
      .names
      .iter()
      .map(|n| ExtendedReferencedExport::Array(vec![n.clone()]))
      .collect()
  }
}

#[cacheable_dyn]
impl ModuleDependency for CssComposeDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn factorize_info(&self) -> std::sync::MutexGuard<'_, FactorizeInfo> {
    self
      .factorize_info
      .lock()
      .expect("dependency factorize_info poisoned")
  }

  fn set_factorize_info(&self, info: FactorizeInfo) {
    *self
      .factorize_info
      .lock()
      .expect("dependency factorize_info poisoned") = info;
  }
}

impl AsDependencyCodeGeneration for CssComposeDependency {}
impl AsContextDependency for CssComposeDependency {}
