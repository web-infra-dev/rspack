//! Invalidations affecting directory walks that are still reading from disk.
use std::sync::{Arc, Mutex};

use rspack_paths::{InternedPath, InternedPathSet};
use rspack_util::fx_hash::FxHashMap;

pub(super) enum Mutation {
  FileTime,
  Unchanged,
  Entry,
  Subtree,
}

#[derive(Default)]
struct Invalidations {
  file_times: InternedPathSet,
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

#[derive(Clone)]
pub(super) struct Scan {
  pub(super) walk: Arc<ActiveWalk>,
  pub(super) root: InternedPath,
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

  pub(super) fn same_subscription(&self, scan: &Scan, context: &InternedPath) -> bool {
    !self.active[&scan.walk.id].subscriptions.contains(context)
  }

  pub(super) fn file_time_was_read(&self, scan: &Scan, path: &InternedPath) -> bool {
    self.active[&scan.walk.id].file_times.contains(path)
  }

  pub(super) fn accepts(&self, scan: &Scan, path: &InternedPath) -> bool {
    !self.active[&scan.walk.id].entries.contains(path) && self.covers_subtree(scan, path)
  }

  pub(super) fn covers_subtree(&self, scan: &Scan, path: &InternedPath) -> bool {
    let invalid = &self.active[&scan.walk.id].subtrees;
    invalid.is_empty()
      || !path
        .ancestors()
        .any(|p| invalid.contains(&InternedPath::from(p)))
  }

  pub(super) fn observe_live(&mut self, path: &InternedPath, mutation: Mutation) {
    for walk in self.active.values_mut() {
      match mutation {
        Mutation::FileTime => {
          walk.file_times.insert(path.clone());
        }
        Mutation::Entry => {
          walk.entries.insert(path.clone());
        }
        Mutation::Subtree => {
          walk.subtrees.insert(path.clone());
        }
        Mutation::Unchanged => {}
      }
    }
  }

  pub(super) fn invalidate_subtree<'a>(
    &mut self,
    path: &InternedPath,
    readers: impl Iterator<Item = &'a Scan> + Clone,
  ) {
    for (id, walk) in &mut self.active {
      if !readers.clone().any(|scan| scan.walk.id == *id) {
        walk.subtrees.insert(path.clone());
      }
    }
  }
}
