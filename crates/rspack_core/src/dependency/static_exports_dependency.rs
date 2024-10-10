use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use swc_core::ecma::atoms::Atom;

use super::AffectType;
use crate::{
  AsContextDependency, AsDependencyTemplate, AsModuleDependency, Dependency, DependencyId,
  DependencyType, ExportNameOrSpec, ExportsOfExportsSpec, ExportsSpec, ModuleGraph,
};

#[cacheable]
#[derive(Debug, Clone)]
pub enum StaticExportsSpec {
  True,
  Array(#[cacheable(with=AsVec<AsPreset>)] Vec<Atom>),
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct StaticExportsDependency {
  id: DependencyId,
  exports: StaticExportsSpec,
  can_mangle: bool,
}

impl StaticExportsDependency {
  pub fn new(exports: StaticExportsSpec, can_mangle: bool) -> Self {
    Self {
      id: DependencyId::new(),
      exports,
      can_mangle,
    }
  }
}

#[cacheable_dyn]
impl Dependency for StaticExportsDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::StaticExports
  }

  fn get_exports(&self, _mg: &ModuleGraph) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: match &self.exports {
        StaticExportsSpec::Array(exports) => ExportsOfExportsSpec::Array(
          exports
            .iter()
            .map(|item| ExportNameOrSpec::String(item.clone()))
            .collect::<Vec<_>>(),
        ),
        StaticExportsSpec::True => ExportsOfExportsSpec::True,
      },
      can_mangle: Some(self.can_mangle),
      ..Default::default()
    })
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::True
  }
}

impl AsDependencyTemplate for StaticExportsDependency {}
impl AsModuleDependency for StaticExportsDependency {}

impl AsContextDependency for StaticExportsDependency {}
