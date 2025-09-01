use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AffectType, AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory,
  DependencyId, DependencyRange, DependencyType, FactorizeInfo, ModuleDependency,
};
use rspack_util::atom::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct RequireEnsureItemDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  range: DependencyRange,
  factorize_info: FactorizeInfo,
}

impl RequireEnsureItemDependency {
  pub fn new(request: Atom, range: DependencyRange) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      range,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for RequireEnsureItemDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RequireEnsureItem
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for RequireEnsureItemDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsDependencyCodeGeneration for RequireEnsureItemDependency {}

impl AsContextDependency for RequireEnsureItemDependency {}
