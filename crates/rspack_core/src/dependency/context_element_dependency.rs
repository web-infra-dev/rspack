use swc_core::ecma::atoms::JsWord;

use crate::{
  AsDependencyTemplate, Context, ContextMode, ContextOptions, Dependency, DependencyCategory,
  DependencyId, DependencyType, ExtendedReferencedExport, ModuleDependency, ModuleGraph,
  ReferencedExport, RuntimeSpec,
};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct ContextElementDependency {
  pub id: DependencyId,
  // TODO remove this async dependency mark
  pub options: ContextOptions,
  pub request: String,
  pub user_request: String,
  pub category: DependencyCategory,
  pub context: Context,
  pub resource_identifier: String,
  pub referenced_exports: Option<Vec<JsWord>>,
}

impl Dependency for ContextElementDependency {
  fn dependency_debug_name(&self) -> &'static str {
    "ContextElementDependency"
  }

  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &self.category
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ContextElement
  }

  fn get_context(&self) -> Option<&Context> {
    Some(&self.context)
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

  fn options(&self) -> Option<&ContextOptions> {
    Some(&self.options)
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
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
