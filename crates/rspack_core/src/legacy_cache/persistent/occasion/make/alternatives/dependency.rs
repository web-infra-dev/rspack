use rspack_cacheable::{cacheable, cacheable_dyn, utils::OwnedOrRef};
#[cfg(allocative)]
use rspack_util::allocative;

use crate::{
  AffectType, AsContextDependency, AsDependencyCodeGeneration, AsModuleDependency, Dependency,
  DependencyId, DependencyRef,
};

#[cacheable]
#[derive(Debug, Default)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct TempDependency {
  id: DependencyId,
}

impl TempDependency {
  pub fn transform_from(dep: OwnedOrRef<DependencyRef>) -> OwnedOrRef<DependencyRef> {
    OwnedOrRef::Owned(DependencyRef::new(TempDependency {
      id: *dep.as_ref().id(),
    }))
  }
}

#[cacheable_dyn]
impl Dependency for TempDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    unreachable!()
  }
}

impl AsModuleDependency for TempDependency {}
impl AsDependencyCodeGeneration for TempDependency {}
impl AsContextDependency for TempDependency {}
