use swc_core::ecma::atoms::Atom;

use crate::{ExtendedReferencedExport, ModuleDependency};
use crate::{ModuleGraph, ReferencedExport, RuntimeSpec};

pub trait ImportDependencyTrait: ModuleDependency {
  fn referenced_exports(&self) -> Option<&Vec<Atom>>;

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    if let Some(referenced_exports) = self.referenced_exports() {
      vec![ReferencedExport::new(referenced_exports.clone(), false).into()]
    } else {
      vec![ExtendedReferencedExport::Array(vec![])]
    }
  }
}
