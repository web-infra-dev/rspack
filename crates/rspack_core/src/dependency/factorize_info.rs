use std::borrow::Cow;

use rspack_cacheable::{cacheable, with::Skip};
use rspack_error::Diagnostic;
use rspack_paths::ArcPath;
use rustc_hash::FxHashSet as HashSet;

use super::BoxDependency;

#[cacheable]
#[derive(Debug, Clone)]
pub enum FactorizeInfo {
  Success,
  Failed {
    file_dependencies: HashSet<ArcPath>,
    context_dependencies: HashSet<ArcPath>,
    missing_dependencies: HashSet<ArcPath>,
    // TODO remove skip after Diagnostic cacheable.
    #[cacheable(with=Skip)]
    diagnostics: Vec<Diagnostic>,
  },
}

impl FactorizeInfo {
  pub fn new(
    diagnostics: Vec<Diagnostic>,
    file_dependencies: HashSet<ArcPath>,
    context_dependencies: HashSet<ArcPath>,
    missing_dependencies: HashSet<ArcPath>,
  ) -> Self {
    if diagnostics.is_empty() {
      Self::Success
    } else {
      Self::Failed {
        file_dependencies,
        context_dependencies,
        missing_dependencies,
        diagnostics,
      }
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

  pub fn get_mut_from(dep: &mut BoxDependency) -> &mut FactorizeInfo {
    //    if let Some(d) = dep.as_context_dependency_mut() {
    //      return d.factorize_info_mut();
    //    }
    if let Some(d) = dep.as_module_dependency_mut() {
      return d.factorize_info_mut();
    }
    panic!(
      "FactorizeInfo::get_mut_from can only be used for context dependency and module dependency"
    )
  }

  pub fn is_failed(&self) -> bool {
    matches!(self, FactorizeInfo::Failed { .. })
  }

  pub fn depends_on(&self, modified_file: &HashSet<ArcPath>) -> bool {
    if let FactorizeInfo::Failed {
      file_dependencies,
      context_dependencies,
      missing_dependencies,
      ..
    } = self
    {
      for item in modified_file {
        if file_dependencies.contains(item)
          || context_dependencies.contains(item)
          || missing_dependencies.contains(item)
        {
          return true;
        }
      }
    }

    false
  }

  pub fn diagnostics(&self) -> Cow<[Diagnostic]> {
    match &self {
      Self::Success => Cow::Owned(vec![]),
      Self::Failed { diagnostics, .. } => Cow::Borrowed(diagnostics),
    }
  }
}

impl Default for FactorizeInfo {
  fn default() -> Self {
    FactorizeInfo::Success
  }
}
