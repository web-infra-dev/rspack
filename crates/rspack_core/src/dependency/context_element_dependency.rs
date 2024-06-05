use swc_core::ecma::atoms::Atom;

use crate::{AsContextDependency, AsDependencyTemplate, Context};
use crate::{ContextMode, ContextOptions, Dependency};
use crate::{DependencyCategory, DependencyId, DependencyType};
use crate::{ExtendedReferencedExport, ModuleDependency};
use crate::{ModuleGraph, ReferencedExport, RuntimeSpec};

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct ContextElementDependency {
  pub id: DependencyId,
  // TODO remove this async dependency mark
  pub options: ContextOptions,
  pub request: String,
  pub user_request: String,
  pub category: DependencyCategory,
  pub context: Context,
  pub resource_identifier: String,
  pub referenced_exports: Option<Vec<Atom>>,
  pub dependency_type: DependencyType,
}

impl Dependency for ContextElementDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &self.category
  }

  fn dependency_type(&self) -> &DependencyType {
    &self.dependency_type
  }

  fn get_context(&self) -> Option<&Context> {
    Some(&self.context)
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }
}

impl ModuleDependency for ContextElementDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.user_request
  }

  fn weak(&self) -> bool {
    matches!(
      self.options.mode,
      ContextMode::AsyncWeak | ContextMode::Weak
    )
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    if let Some(referenced_exports) = &self.referenced_exports {
      vec![ReferencedExport::new(referenced_exports.clone(), false).into()]
    } else {
      vec![ExtendedReferencedExport::Array(vec![])]
    }
  }
}

impl AsDependencyTemplate for ContextElementDependency {}
impl AsContextDependency for ContextElementDependency {}
