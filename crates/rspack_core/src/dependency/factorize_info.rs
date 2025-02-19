use std::borrow::Cow;

use rspack_cacheable::{cacheable, with::Skip};
use rspack_error::Diagnostic;
use rspack_paths::ArcPath;
use rustc_hash::FxHashSet as HashSet;

use super::{BoxDependency, DependencyId};

#[cacheable]
#[derive(Debug, Clone, Default)]
pub enum FactorizeInfo {
  #[default]
  Success,
  Failed {
    related_dep_ids: Vec<DependencyId>,
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
    related_dep_ids: Vec<DependencyId>,
    file_dependencies: HashSet<ArcPath>,
    context_dependencies: HashSet<ArcPath>,
    missing_dependencies: HashSet<ArcPath>,
  ) -> Self {
    if diagnostics.is_empty() {
      Self::Success
    } else {
      Self::Failed {
        related_dep_ids,
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

  pub fn is_success(&self) -> bool {
    matches!(self, FactorizeInfo::Success)
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

  pub fn related_dep_ids(&self) -> Cow<[DependencyId]> {
    match &self {
      Self::Success => Cow::Owned(vec![]),
      Self::Failed {
        related_dep_ids, ..
      } => Cow::Borrowed(related_dep_ids),
    }
  }

  pub fn file_dependencies(&self) -> Cow<HashSet<ArcPath>> {
    match &self {
      Self::Success => Cow::Owned(Default::default()),
      Self::Failed {
        file_dependencies, ..
      } => Cow::Borrowed(file_dependencies),
    }
  }

  pub fn context_dependencies(&self) -> Cow<HashSet<ArcPath>> {
    match &self {
      Self::Success => Cow::Owned(Default::default()),
      Self::Failed {
        context_dependencies,
        ..
      } => Cow::Borrowed(context_dependencies),
    }
  }

  pub fn missing_dependencies(&self) -> Cow<HashSet<ArcPath>> {
    match &self {
      Self::Success => Cow::Owned(Default::default()),
      Self::Failed {
        missing_dependencies,
        ..
      } => Cow::Borrowed(missing_dependencies),
    }
  }

  pub fn diagnostics(&self) -> Cow<[Diagnostic]> {
    match &self {
      Self::Success => Cow::Owned(vec![]),
      Self::Failed { diagnostics, .. } => Cow::Borrowed(diagnostics),
    }
  }
}
