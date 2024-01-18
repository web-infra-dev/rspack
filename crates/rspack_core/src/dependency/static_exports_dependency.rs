use swc_core::ecma::atoms::Atom;

use crate::{
  AsContextDependency, AsDependencyTemplate, AsModuleDependency, Dependency, DependencyId,
  DependencyType, ExportNameOrSpec, ExportsOfExportsSpec, ExportsSpec, ModuleGraph,
};

#[derive(Debug, Clone)]
pub struct StaticExportsDependency {
  id: DependencyId,
  exports: Vec<Atom>,
  can_mangle: bool,
}

impl StaticExportsDependency {
  pub fn new(exports: Vec<Atom>, can_mangle: bool) -> Self {
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

  fn dependency_debug_name(&self) -> &'static str {
    "StaticExportsDependency"
  }

  fn get_exports(&self, _mg: &ModuleGraph) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Array(
        self
          .exports
          .iter()
          .map(|item| ExportNameOrSpec::String(item.clone()))
          .collect::<Vec<_>>(),
      ),
      can_mangle: Some(self.can_mangle),
      ..Default::default()
    })
  }
}

impl AsDependencyTemplate for StaticExportsDependency {}
impl AsModuleDependency for StaticExportsDependency {}

impl AsContextDependency for StaticExportsDependency {}
