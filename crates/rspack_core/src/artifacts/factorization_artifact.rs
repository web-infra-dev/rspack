use rspack_cacheable::cacheable;
use rspack_error::Diagnostic;
use rspack_paths::ArcPathSet;
use rustc_hash::FxHashMap;

use crate::DependencyId;

#[cacheable]
#[derive(Debug, Clone)]
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
    assert!(
      !related_dep_ids.is_empty(),
      "factorization should contain at least one dependency"
    );
    Self {
      related_dep_ids,
      file_dependencies,
      context_dependencies,
      missing_dependencies,
      diagnostics,
    }
  }

  pub fn owner_dep_id(&self) -> DependencyId {
    self.related_dep_ids[0]
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

#[derive(Debug, Default)]
pub(crate) struct FactorizationArtifact {
  infos: FxHashMap<DependencyId, FactorizeInfo>,
  dependency_owners: FxHashMap<DependencyId, DependencyId>,
}

impl FactorizationArtifact {
  pub(crate) fn insert(&mut self, info: FactorizeInfo) {
    let owner_dep_id = info.owner_dep_id();

    if let Some(previous) = self.infos.remove(&owner_dep_id) {
      for dep_id in previous.related_dep_ids() {
        self.dependency_owners.remove(dep_id);
      }
    }

    for dep_id in info.related_dep_ids() {
      let previous_owner = self.dependency_owners.insert(*dep_id, owner_dep_id);
      debug_assert!(
        previous_owner.is_none() || previous_owner == Some(owner_dep_id),
        "dependency should only belong to one factorization"
      );
    }
    self.infos.insert(owner_dep_id, info);
  }

  pub(crate) fn get(&self, dep_id: &DependencyId) -> Option<&FactorizeInfo> {
    let owner_dep_id = self.dependency_owners.get(dep_id)?;
    self.infos.get(owner_dep_id)
  }

  pub(crate) fn get_by_owner(&self, dep_id: &DependencyId) -> Option<&FactorizeInfo> {
    self.infos.get(dep_id)
  }

  pub(crate) fn revoke(&mut self, dep_id: &DependencyId) -> Option<(DependencyId, FactorizeInfo)> {
    let owner_dep_id = *self.dependency_owners.get(dep_id)?;
    let info = self
      .infos
      .remove(&owner_dep_id)
      .expect("factorization owner should have info");
    for related_dep_id in info.related_dep_ids() {
      self.dependency_owners.remove(related_dep_id);
    }
    Some((owner_dep_id, info))
  }

  pub(crate) fn iter(&self) -> impl Iterator<Item = (DependencyId, &FactorizeInfo)> {
    self.infos.iter().map(|(dep_id, info)| (*dep_id, info))
  }
}
