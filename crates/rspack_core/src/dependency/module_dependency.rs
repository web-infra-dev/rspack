use dyn_clone::clone_trait_object;

use super::Dependency;
use crate::{DependencyCondition, ExtendedReferencedExport, ModuleGraph, RuntimeSpec};

pub trait ModuleDependency: Dependency {
  fn request(&self) -> &str;
  fn user_request(&self) -> &str;
  // TODO: move to ModuleGraphConnection
  fn weak(&self) -> bool {
    false
  }
  fn set_request(&mut self, request: String);

  fn get_optional(&self) -> bool {
    false
  }

  fn get_condition(&self) -> Option<DependencyCondition> {
    None
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    vec![ExtendedReferencedExport::Array(vec![])]
  }

  fn is_export_all(&self) -> Option<bool> {
    None
  }
}

clone_trait_object!(ModuleDependency);

pub trait AsModuleDependency {
  fn as_module_dependency(&self) -> Option<&dyn ModuleDependency> {
    None
  }

  fn as_module_dependency_mut(&mut self) -> Option<&mut dyn ModuleDependency> {
    None
  }
}

impl<T: ModuleDependency> AsModuleDependency for T {
  fn as_module_dependency(&self) -> Option<&dyn ModuleDependency> {
    Some(self)
  }

  fn as_module_dependency_mut(&mut self) -> Option<&mut dyn ModuleDependency> {
    Some(self)
  }
}
