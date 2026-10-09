use std::collections::hash_map::Entry;

use rspack_cacheable::cacheable;
use rspack_error::Diagnostic;
use rspack_paths::{InternedPath, InternedPathSet, SharedPathList};
use rustc_hash::FxHashMap;

use crate::DependencyId;

#[cacheable]
#[derive(Debug, Clone)]
pub struct FactorizeInfo {
  related_dep_ids: Vec<DependencyId>,
  file_dependencies: SharedPathList,
  context_dependencies: SharedPathList,
  missing_dependencies: SharedPathList,
  diagnostics: Vec<Diagnostic>,
}

impl FactorizeInfo {
  pub fn new(
    diagnostics: Vec<Diagnostic>,
    related_dep_ids: Vec<DependencyId>,
    file_dependencies: InternedPathSet,
    context_dependencies: InternedPathSet,
    missing_dependencies: InternedPathSet,
  ) -> Self {
    assert!(
      !related_dep_ids.is_empty(),
      "factorization should contain at least one dependency"
    );
    Self {
      related_dep_ids,
      file_dependencies: file_dependencies.into_iter().collect(),
      context_dependencies: context_dependencies.into_iter().collect(),
      missing_dependencies: missing_dependencies.into_iter().collect(),
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

  pub fn file_dependencies(&self) -> &[InternedPath] {
    self.file_dependencies.as_slice()
  }

  pub fn context_dependencies(&self) -> &[InternedPath] {
    self.context_dependencies.as_slice()
  }

  pub fn missing_dependencies(&self) -> &[InternedPath] {
    self.missing_dependencies.as_slice()
  }

  pub fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }
}

#[derive(Debug, Default)]
pub(crate) struct FactorizationArtifact {
  infos: FxHashMap<DependencyId, FactorizeInfo>,
  dependency_owners: FxHashMap<DependencyId, DependencyId>,
  /// Canonical lists and the number of fields in `infos` using each one.
  /// Only serial insertion/revocation changes this index. Counts exclude
  /// external clones, which keep their data alive independently through Arc.
  path_lists: FxHashMap<SharedPathList, usize>,
}

impl FactorizationArtifact {
  pub(crate) fn insert(&mut self, mut info: FactorizeInfo) {
    let owner_dep_id = info.owner_dep_id();

    self.revoke(&owner_dep_id);

    // Both background factorization and parallel cache decoding produce owned
    // lists. Deduplicate only when their results reach this serial owner.
    for paths in [
      &mut info.file_dependencies,
      &mut info.context_dependencies,
      &mut info.missing_dependencies,
    ] {
      if paths.is_empty() {
        continue;
      }
      match self.path_lists.entry(paths.clone()) {
        Entry::Occupied(mut entry) => {
          *entry.get_mut() += 1;
          *paths = entry.key().clone();
        }
        Entry::Vacant(entry) => {
          entry.insert(1);
        }
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
    for paths in [
      &info.file_dependencies,
      &info.context_dependencies,
      &info.missing_dependencies,
    ] {
      if paths.is_empty() {
        continue;
      }
      let references = self
        .path_lists
        .get_mut(paths)
        .expect("factorization paths should be indexed");
      *references -= 1;
      if *references == 0 {
        self.path_lists.remove(paths);
      }
    }
    Some((owner_dep_id, info))
  }

  pub(crate) fn iter(&self) -> impl Iterator<Item = (DependencyId, &FactorizeInfo)> {
    self.infos.iter().map(|(dep_id, info)| (*dep_id, info))
  }
}
