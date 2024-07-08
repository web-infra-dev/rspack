use swc_core::ecma::atoms::Atom;

use crate::{
  AsContextDependency, AsDependencyTemplate, AsModuleDependency, Dependency, DependencyId,
  DependencyType, ExportNameOrSpec, ExportsOfExportsSpec, ExportsSpec, ModuleGraph,
};

#[derive(Debug, Clone)]
pub enum StaticExportsSpec {
  True,
  Array(Vec<Atom>),
}

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
}

impl AsDependencyTemplate for StaticExportsDependency {}
impl AsModuleDependency for StaticExportsDependency {}

impl AsContextDependency for StaticExportsDependency {}
