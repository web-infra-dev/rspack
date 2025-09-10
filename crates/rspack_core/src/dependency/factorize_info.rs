use std::borrow::Cow;

use rspack_cacheable::cacheable;
use rspack_error::Diagnostic;
use rspack_paths::ArcPath;
use rustc_hash::FxHashSet as HashSet;

use super::DependencyId;

#[cacheable]
#[derive(Debug, Clone)]
pub enum FactorizeInfo {
  Success {
    related_dep_ids: Vec<DependencyId>,
    missing_dependencies: HashSet<ArcPath>,
  },
  Failed {
    related_dep_ids: Vec<DependencyId>,
    file_dependencies: HashSet<ArcPath>,
    context_dependencies: HashSet<ArcPath>,
    missing_dependencies: HashSet<ArcPath>,
    diagnostics: Vec<Diagnostic>,
  },
}

impl Default for FactorizeInfo {
  fn default() -> Self {
    Self::Success {
      related_dep_ids: Default::default(),
      missing_dependencies: Default::default(),
    }
  }
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
      Self::Success {
        related_dep_ids,
        missing_dependencies,
      }
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

  pub fn is_success(&self) -> bool {
    matches!(self, FactorizeInfo::Success { .. })
  }

  pub fn depends_on(&self, modified_file: &HashSet<ArcPath>) -> bool {
    match self {
      FactorizeInfo::Success {
        missing_dependencies,
        ..
      } => missing_dependencies
        .intersection(modified_file)
        .next()
        .is_some(),
      FactorizeInfo::Failed {
        file_dependencies,
        context_dependencies,
        missing_dependencies,
        ..
      } => {
        for item in modified_file {
          if file_dependencies.contains(item)
            || context_dependencies.contains(item)
            || missing_dependencies.contains(item)
          {
            return true;
          }
        }
        false
      }
    }
  }

  pub fn related_dep_ids(&self) -> &[DependencyId] {
    match &self {
      Self::Success {
        related_dep_ids, ..
      } => related_dep_ids,
      Self::Failed {
        related_dep_ids, ..
      } => related_dep_ids,
    }
  }

  pub fn file_dependencies(&self) -> Cow<'_, HashSet<ArcPath>> {
    match &self {
      Self::Success { .. } => Cow::Owned(Default::default()),
      Self::Failed {
        file_dependencies, ..
      } => Cow::Borrowed(file_dependencies),
    }
  }

  pub fn context_dependencies(&self) -> Cow<'_, HashSet<ArcPath>> {
    match &self {
      Self::Success { .. } => Cow::Owned(Default::default()),
      Self::Failed {
        context_dependencies,
        ..
      } => Cow::Borrowed(context_dependencies),
    }
  }

  pub fn missing_dependencies(&self) -> Cow<'_, HashSet<ArcPath>> {
    match &self {
      Self::Success {
        missing_dependencies,
        ..
      } => Cow::Borrowed(missing_dependencies),
      Self::Failed {
        missing_dependencies,
        ..
      } => Cow::Borrowed(missing_dependencies),
    }
  }

  pub fn diagnostics(&self) -> Cow<'_, [Diagnostic]> {
    match &self {
      Self::Success { .. } => Cow::Owned(vec![]),
      Self::Failed { diagnostics, .. } => Cow::Borrowed(diagnostics),
    }
  }
}
