use rspack_cacheable::{cacheable, cacheable_dyn, utils::OwnedOrRef};

use crate::{
  AffectType, AsContextDependency, AsDependencyCodeGeneration, AsModuleDependency, BoxDependency,
  Dependency, DependencyId,
};

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct TempDependency {
  id: DependencyId,
}

impl TempDependency {
  #[allow(clippy::needless_pass_by_value)]
  pub fn transform_from(dep: OwnedOrRef<BoxDependency>) -> OwnedOrRef<BoxDependency> {
    OwnedOrRef::Owned(Box::new(TempDependency {
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
