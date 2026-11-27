//! Invalidations affecting directory walks that are still reading from disk.
use std::sync::{Arc, Mutex};

use rspack_paths::{InternedPath, InternedPathSet};
use rspack_util::fx_hash::{FxHashMap, FxHashSet};

use crate::paths::RegistrationKind;

pub(super) enum Mutation {
  Metadata,
  Entry,
  Subtree,
}

#[derive(Default)]
struct Invalidations {
  metadata_rechecks: InternedPathSet,
  entries: InternedPathSet,
  subtrees: InternedPathSet,
  registrations: FxHashSet<(InternedPath, RegistrationKind)>,
}

#[derive(Default)]
pub(super) struct Observations {
  next_walk: u64,
  active: FxHashMap<u64, Invalidations>,
}

pub(super) struct ActiveWalk {
  observations: Arc<Mutex<Observations>>,
  id: u64,
}

impl ActiveWalk {
  pub(super) fn start(observations: &Arc<Mutex<Observations>>) -> Arc<Self> {
    let mut state = observations.lock().expect("watch observations poisoned");
    let id = state.next_walk;
    state.next_walk += 1;
    state.active.insert(id, Invalidations::default());
    Arc::new(Self {
      observations: Arc::clone(observations),
      id,
    })
  }
}

impl Drop for ActiveWalk {
  fn drop(&mut self) {
    self
      .observations
      .lock()
      .expect("watch observations poisoned")
      .active
      .remove(&self.id);
  }
}

impl Observations {
  pub(super) fn registration_changed(&mut self, path: &InternedPath, kind: RegistrationKind) {
    for walk in self.active.values_mut() {
      walk.registrations.insert((path.clone(), kind));
      if kind != RegistrationKind::Directory {
        walk.metadata_rechecks.insert(path.clone());
      }
    }
  }
  pub(super) fn same_registration(
    &self,
    walk: &ActiveWalk,
    path: &InternedPath,
    kind: RegistrationKind,
  ) -> bool {
    !self.active[&walk.id]
      .registrations
      .contains(&(path.clone(), kind))
  }

  pub(super) fn needs_metadata_recheck(&self, walk: &ActiveWalk, path: &InternedPath) -> bool {
    self.active[&walk.id].metadata_rechecks.contains(path)
  }

  pub(super) fn take_metadata_recheck(&mut self, walk: &ActiveWalk, path: &InternedPath) -> bool {
    self
      .active
      .get_mut(&walk.id)
      .expect("active observation")
      .metadata_rechecks
      .remove(path)
  }

  pub(super) fn accepts(&self, walk: &ActiveWalk, path: &InternedPath) -> bool {
    !self.active[&walk.id].entries.contains(path) && self.covers_subtree(walk, path)
  }

  pub(super) fn covers_subtree(&self, walk: &ActiveWalk, path: &InternedPath) -> bool {
    let invalid = &self.active[&walk.id].subtrees;
    invalid.is_empty()
      || !path
        .ancestors()
        .any(|p| invalid.contains(&InternedPath::from(p)))
  }

  pub(super) fn observe_live(&mut self, path: &InternedPath, mutation: Mutation) {
    for walk in self.active.values_mut() {
      match mutation {
        Mutation::Metadata => {
          walk.metadata_rechecks.insert(path.clone());
        }
        Mutation::Entry => {
          walk.entries.insert(path.clone());
        }
        Mutation::Subtree => {
          walk.subtrees.insert(path.clone());
        }
      }
    }
  }
}
