use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AffectType, AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory,
  DependencyId, DependencyRange, DependencyType, ModuleDependency,
};
use rspack_util::atom::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct RequireEnsureItemDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  range: DependencyRange,
}

impl RequireEnsureItemDependency {
  pub fn new(request: Atom, range: DependencyRange) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      range,
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

  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
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
}

impl AsDependencyTemplate for RequireEnsureItemDependency {}

impl AsContextDependency for RequireEnsureItemDependency {}
