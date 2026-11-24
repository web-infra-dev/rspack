//! Versions are needed only while an older scan can still commit or report an event.
use std::{
  collections::BTreeSet,
  sync::{Arc, Mutex},
};

use rspack_paths::{InternedPath, InternedPathMap};

pub(super) enum Mutation {
  Unchanged,
  Entry,
  Subtree,
}

#[derive(Default)]
pub(super) struct Observations {
  sequence: u64,
  active: BTreeSet<u64>,
  entries: InternedPathMap<u64>,
  subtrees: InternedPathMap<u64>,
  unregistered: InternedPathMap<u64>,
}

pub(super) struct ScanEpoch {
  observations: Arc<Mutex<Observations>>,
  sequence: u64,
}

#[derive(Clone)]
pub(super) struct Scan {
  pub(super) epoch: Arc<ScanEpoch>,
  pub(super) root: InternedPath,
}

impl ScanEpoch {
  pub(super) fn start(observations: &Arc<Mutex<Observations>>) -> Arc<Self> {
    let mut state = observations.lock().expect("watch observations poisoned");
    let sequence = state.advance();
    state.active.insert(sequence);
    Arc::new(Self {
      observations: Arc::clone(observations),
      sequence,
    })
  }
}

impl Drop for ScanEpoch {
  fn drop(&mut self) {
    let mut state = self
      .observations
      .lock()
      .expect("watch observations poisoned");
    state.active.remove(&self.sequence);
    let Some(&oldest) = state.active.first() else {
      *state = Observations::default();
      return;
    };
    if self.sequence < oldest {
      state.entries.retain(|_, version| *version > oldest);
      state.subtrees.retain(|_, version| *version > oldest);
      state.unregistered.retain(|_, version| *version > oldest);
    }
  }
}

impl Observations {
  fn advance(&mut self) -> u64 {
    self.sequence += 1;
    self.sequence
  }

  pub(super) fn unregister(&mut self, path: &InternedPath) {
    if !self.active.is_empty() {
      let version = self.advance();
      self.unregistered.insert(path.clone(), version);
    }
  }

  pub(super) fn accepts(&self, scan: &Scan, path: &InternedPath) -> bool {
    let version = scan.epoch.sequence;
    self.entries.get(path).is_none_or(|v| *v <= version) && self.covers_subtree(scan, path)
  }

  pub(super) fn covers_subtree(&self, scan: &Scan, path: &InternedPath) -> bool {
    let version = scan.epoch.sequence;
    self
      .unregistered
      .get(&scan.root)
      .is_none_or(|v| *v <= version)
      && (self.subtrees.is_empty()
        || !path.ancestors().any(|ancestor| {
          self
            .subtrees
            .get(&InternedPath::from(ancestor))
            .is_some_and(|v| *v > version)
        }))
  }

  pub(super) fn observe(&mut self, path: &InternedPath, scan: Option<&Scan>, mutation: Mutation) {
    if matches!(mutation, Mutation::Unchanged) {
      return;
    }
    let Some(&oldest) = self.active.first() else {
      return;
    };
    let version = scan.map_or_else(|| self.advance(), |scan| scan.epoch.sequence);
    // A scan need only leave a version for an older scan, never for itself.
    if version > oldest {
      match mutation {
        Mutation::Unchanged => {}
        Mutation::Subtree => {
          self.subtrees.insert(path.clone(), version);
        }
        Mutation::Entry => {
          self.entries.insert(path.clone(), version);
        }
      }
    }
  }
}
