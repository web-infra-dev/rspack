use std::borrow::Cow;

use rspack_cacheable::{cacheable, with::Skip};
use rspack_error::Diagnostic;
use rspack_paths::ArcPath;
use rustc_hash::FxHashSet as HashSet;

use super::{BoxDependency, DependencyId};

/// Factorize info for dependency
#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct FactorizeInfo {
  /// The dependency ids which have same origin module and target module.
  ///
  /// The process dependencies task in make will merge the dependencies which will has
  /// same origin module and target module as a single factorize task, and we will add
  /// the factorize info to the first dependency and use this field to track related dependencies.
  related_dep_ids: Vec<DependencyId>,
  /// The files that current dependency depends on.
  file_dependencies: HashSet<ArcPath>,
  /// The directory that current dependency depends on.
  context_dependencies: HashSet<ArcPath>,
  /// The missing files that current dependency depends on.
  missing_dependencies: HashSet<ArcPath>,
  // TODO remove Skip and Option after Diagnostic cacheable.
  /// The diagnostics generate by factorize task.
  #[cacheable(with=Skip)]
  diagnostics: Option<Vec<Diagnostic>>,
}

impl FactorizeInfo {
  pub fn new(
    related_dep_ids: Vec<DependencyId>,
    file_dependencies: HashSet<ArcPath>,
    context_dependencies: HashSet<ArcPath>,
    missing_dependencies: HashSet<ArcPath>,
    diagnostics: Vec<Diagnostic>,
  ) -> Self {
    let diagnostics = if diagnostics.is_empty() {
      None
    } else {
      Some(diagnostics)
    };
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

  // TODO remove it after diagnostic cacheable.
  pub fn is_success(&self) -> bool {
    self.diagnostics.is_none()
  }

  pub fn depends_on(&self, modified_file: &HashSet<ArcPath>) -> bool {
    for item in modified_file {
      if self.file_dependencies.contains(item)
        || self.context_dependencies.contains(item)
        || self.missing_dependencies.contains(item)
      {
        return true;
      }
    }
    false
  }

  pub fn related_dep_ids(&self) -> &[DependencyId] {
    &self.related_dep_ids
  }

  pub fn file_dependencies(&self) -> &HashSet<ArcPath> {
    &self.file_dependencies
  }

  pub fn context_dependencies(&self) -> &HashSet<ArcPath> {
    &self.context_dependencies
  }

  pub fn missing_dependencies(&self) -> &HashSet<ArcPath> {
    &self.missing_dependencies
  }

  pub fn diagnostics(&self) -> Cow<[Diagnostic]> {
    match &self.diagnostics {
      Some(d) => Cow::Borrowed(d),
      None => Cow::Owned(vec![]),
    }
  }
}
