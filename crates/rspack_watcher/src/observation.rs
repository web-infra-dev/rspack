//! Versions are needed only while an older scan can still commit or report an event.
use std::{
  collections::BTreeSet,
  sync::{Arc, Mutex},
  time::SystemTime,
};

use rspack_paths::{InternedPath, InternedPathMap};

#[derive(Clone, Copy)]
pub(super) enum Registration {
  File,
  Context,
  Missing,
}

pub(super) enum Mutation {
  Unchanged,
  Entry,
  Subtree,
  // A directory walk supplies membership; a single-path check may owe the build a notification.
  Scan(Option<SystemTime>),
}

#[derive(Default)]
pub(super) struct Observations {
  sequence: u64,
  active: BTreeSet<u64>,
  entries: InternedPathMap<u64>,
  subtrees: InternedPathMap<u64>,
  scans: [InternedPathMap<(u64, Option<SystemTime>)>; 2],
  registration_changes: [InternedPathMap<u64>; 3],
}

pub(super) struct ScanEpoch {
  observations: Arc<Mutex<Observations>>,
  sequence: u64,
}

#[derive(Clone)]
pub(super) struct Scan {
  pub(super) epoch: Arc<ScanEpoch>,
  pub(super) kind: Registration,
  pub(super) root: InternedPath,
  pub(super) recursive: bool,
}

impl Scan {
  pub(super) fn new(epoch: &Arc<ScanEpoch>, kind: Registration, root: &InternedPath) -> Self {
    Self {
      epoch: Arc::clone(epoch),
      kind,
      root: root.clone(),
      recursive: false,
    }
  }
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
      for scans in &mut state.scans {
        scans.retain(|_, (version, _)| *version > oldest);
      }
      state.subtrees.retain(|_, version| *version > oldest);
      for registrations in &mut state.registration_changes {
        registrations.retain(|_, version| *version > oldest);
      }
    }
  }
}

impl Observations {
  fn advance(&mut self) -> u64 {
    self.sequence += 1;
    self.sequence
  }

  pub(super) fn registration_changed(&mut self, path: &InternedPath, kind: Registration) {
    if !self.active.is_empty() {
      let version = self.advance();
      self.registration_changes[kind as usize].insert(path.clone(), version);
    }
  }

  pub(super) fn same_subscription(&self, scan: &Scan, path: &InternedPath) -> bool {
    self.registration_changes[scan.kind as usize]
      .get(path)
      .is_none_or(|v| *v <= scan.epoch.sequence)
  }

  pub(super) fn accepts(&self, scan: &Scan, path: &InternedPath) -> bool {
    let version = scan.epoch.sequence;
    self.scans[usize::from(scan.recursive)]
      .get(path)
      .is_none_or(|(v, _)| *v <= version)
      && self.entries.get(path).is_none_or(|v| *v <= version)
      && self.covers_subtree(scan, path)
  }

  pub(super) fn covers_subtree(&self, scan: &Scan, path: &InternedPath) -> bool {
    let version = scan.epoch.sequence;
    self.subtrees.is_empty()
      || !path.ancestors().any(|ancestor| {
        self
          .subtrees
          .get(&InternedPath::from(ancestor))
          .is_some_and(|v| *v > version)
      })
  }

  pub(super) fn newer_scan(
    &self,
    path: &InternedPath,
    scan: &Scan,
  ) -> Option<(u64, Option<SystemTime>)> {
    self.scans[usize::from(!scan.recursive)]
      .get(path)
      .filter(|(version, _)| *version > scan.epoch.sequence)
      .copied()
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
        Mutation::Scan(time) => {
          let scan = scan.expect("a scan records its own observation");
          self.scans[usize::from(scan.recursive)].insert(path.clone(), (version, time));
        }
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
