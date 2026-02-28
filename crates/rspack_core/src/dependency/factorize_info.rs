use std::sync::{Mutex, OnceLock};

use rspack_cacheable::cacheable;
use rspack_error::Diagnostic;
use rspack_paths::ArcPathSet;
use rustc_hash::FxHashMap;

use super::{ArcDependency, Dependency, DependencyId};

static FACTORIZE_INFO_MAP: OnceLock<Mutex<FxHashMap<DependencyId, FactorizeInfo>>> =
  OnceLock::new();

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

  pub fn get_from(dep: &dyn Dependency) -> Option<FactorizeInfo> {
    let map = FACTORIZE_INFO_MAP.get_or_init(|| Mutex::new(FxHashMap::default()));
    if let Some(info) = map
      .lock()
      .expect("factorize info map poisoned")
      .get(dep.id())
      .cloned()
    {
      return Some(info);
    }
    if let Some(d) = dep.as_context_dependency() {
      Some(d.factorize_info().clone())
    } else if let Some(d) = dep.as_module_dependency() {
      Some(d.factorize_info().clone())
    } else {
      None
    }
  }

  pub(crate) fn revoke_arc(dep: &ArcDependency) -> Option<FactorizeInfo> {
    let map = FACTORIZE_INFO_MAP.get_or_init(|| Mutex::new(FxHashMap::default()));
    map
      .lock()
      .expect("factorize info map poisoned")
      .remove(dep.id())
  }

  pub(crate) fn set_arc(dep: &ArcDependency, info: FactorizeInfo) -> Option<()> {
    let map = FACTORIZE_INFO_MAP.get_or_init(|| Mutex::new(FxHashMap::default()));
    map
      .lock()
      .expect("factorize info map poisoned")
      .insert(*dep.id(), info);
    Some(())
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
