//! Invalidations affecting directory walks that are still reading from disk.
use std::sync::{Arc, Mutex};

use rspack_paths::{InternedPath, InternedPathSet};
use rspack_util::fx_hash::FxHashMap;

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
  subscriptions: InternedPathSet,
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
  pub(super) fn context_changed(&mut self, path: &InternedPath) {
    for walk in self.active.values_mut() {
      walk.subscriptions.insert(path.clone());
    }
  }

  pub(super) fn same_subscription(&self, walk: &ActiveWalk, context: &InternedPath) -> bool {
    !self.active[&walk.id].subscriptions.contains(context)
  }

  pub(super) fn needs_metadata_recheck(&self, walk: &ActiveWalk, path: &InternedPath) -> bool {
    self.active[&walk.id].metadata_rechecks.contains(path)
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

  pub(super) fn invalidate(
    &mut self,
    path: &InternedPath,
    mutation: Mutation,
    source: Option<&ActiveWalk>,
  ) {
    for (id, walk) in &mut self.active {
      if source.is_some_and(|source| source.id == *id) {
        continue;
      }
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
