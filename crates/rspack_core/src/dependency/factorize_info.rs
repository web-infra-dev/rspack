use rspack_cacheable::cacheable;
use rspack_error::Diagnostic;
use rspack_paths::ArcPathSet;

use super::{BoxDependency, DependencyId};

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct FactorizeInfo {
  related_dep_ids: Vec<DependencyId>,
  file_dependencies: ArcPathSet,
  context_dependencies: ArcPathSet,
  missing_dependencies: ArcPathSet,
  diagnostics: Vec<Diagnostic>,
}

impl FactorizeInfo {
  pub fn new(
    diagnostics: Vec<Diagnostic>,
    related_dep_ids: Vec<DependencyId>,
    file_dependencies: ArcPathSet,
    context_dependencies: ArcPathSet,
    missing_dependencies: ArcPathSet,
  ) -> Self {
    Self {
      related_dep_ids,
      file_dependencies,
      context_dependencies,
      missing_dependencies,
      diagnostics,
    }
  }

  pub fn get_from(dep: &BoxDependency) -> Option<&FactorizeInfo> {
    if let Some(d) = dep.as_context_dependency() {
      Some(d.factorize_info())
    } else if let Some(d) = dep.as_module_dependency() {
      Some(d.factorize_info())
    } else {
      None
    }
  }

  pub fn revoke(dep: &mut BoxDependency) -> Option<FactorizeInfo> {
    if let Some(d) = dep.as_context_dependency_mut() {
      Some(std::mem::take(d.factorize_info_mut()))
    } else if let Some(d) = dep.as_module_dependency_mut() {
      Some(std::mem::take(d.factorize_info_mut()))
    } else {
      None
    }
  }

  pub fn is_success(&self) -> bool {
    self.diagnostics.is_empty()
  }

  pub fn related_dep_ids(&self) -> &[DependencyId] {
    &self.related_dep_ids
  }

  pub fn file_dependencies(&self) -> &ArcPathSet {
    &self.file_dependencies
  }

  pub fn context_dependencies(&self) -> &ArcPathSet {
    &self.context_dependencies
  }

  pub fn missing_dependencies(&self) -> &ArcPathSet {
    &self.missing_dependencies
  }

  pub fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }
}
