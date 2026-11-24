use std::sync::{Arc, Mutex, MutexGuard};

use rspack_paths::{InternedPath, InternedPathDashMap, InternedPathMap};

use crate::observation::Observations;

pub(super) enum MemberKind {
  File,
  Directory,
}

// An initial scan can follow a directory symlink that a live event records as a file.
#[derive(Clone, Copy, Default)]
pub(super) struct ContextMember {
  pub(super) file: bool,
  pub(super) directory: bool,
}

#[derive(Clone, Default)]
pub(super) struct DirectorySubscriptions {
  pub(super) explicit: bool,
  pub(super) parent: Option<InternedPath>,
}

impl DirectorySubscriptions {
  fn is_active(&self) -> bool {
    self.explicit || self.parent.is_some()
  }
}

pub(super) trait ContextLookup {
  fn subscriptions(&self, path: &InternedPath) -> DirectorySubscriptions;
}

#[derive(Default)]
pub(super) struct DirectoryWatcher {
  pub(super) members: InternedPathMap<ContextMember>,
  subscriptions: DirectorySubscriptions,
}

impl DirectoryWatcher {
  fn is_unused(&self) -> bool {
    self.members.is_empty() && !self.subscriptions.is_active()
  }
}

#[derive(Default)]
pub(super) struct WatcherManager {
  pub(super) directories: InternedPathDashMap<DirectoryWatcher>,
  pub(super) observations: Arc<Mutex<Observations>>,
}

impl WatcherManager {
  pub(super) fn lock_observations(&self) -> MutexGuard<'_, Observations> {
    self
      .observations
      .lock()
      .expect("watch observations poisoned")
  }

  pub(super) fn subscribe_context(&self, path: &InternedPath) {
    let mut watcher = self.directories.entry(path.clone()).or_default();
    watcher.subscriptions.explicit = true;
  }

  pub(super) fn unsubscribe_context(&self, path: &InternedPath) {
    let mut state = self.lock_observations();
    state.unregister(path);
    self.directories.remove_if_mut(path, |_, watcher| {
      watcher.subscriptions.explicit = false;
      watcher.is_unused()
    });
  }

  pub(super) fn is_explicit_context(&self, path: &InternedPath) -> bool {
    self
      .directories
      .get(path)
      .is_some_and(|watcher| watcher.subscriptions.explicit)
  }

  pub(super) fn is_watched_directory(&self, path: &InternedPath) -> bool {
    self
      .directories
      .get(path)
      .is_some_and(|watcher| watcher.subscriptions.is_active())
  }

  pub(super) fn add_member(&self, path: &InternedPath, kind: MemberKind) -> bool {
    let parent = InternedPath::from(path.parent().expect("a context member has a parent"));
    let mut watcher = self.directories.entry(parent).or_default();
    let member = watcher.members.entry(path.clone()).or_default();
    match kind {
      MemberKind::File => !std::mem::replace(&mut member.file, true),
      MemberKind::Directory => {
        let added = !std::mem::replace(&mut member.directory, true);
        let parent = watcher.key().clone();
        drop(watcher);
        let mut child = self.directories.entry(path.clone()).or_default();
        child.subscriptions.parent = Some(parent);
        added
      }
    }
  }

  pub(super) fn member(&self, path: &InternedPath) -> Option<ContextMember> {
    let parent = InternedPath::from(path.parent()?);
    self.directories.get(&parent)?.members.get(path).copied()
  }

  pub(super) fn remove_member(&self, path: &InternedPath) -> ContextMember {
    let mut removed = ContextMember::default();
    if let Some(parent) = path.parent() {
      self
        .directories
        .remove_if_mut(&InternedPath::from(parent), |_, watcher| {
          removed = watcher.members.remove(path).unwrap_or_default();
          watcher.is_unused()
        });
    }
    if removed.directory {
      self.unsubscribe_parent(path);
    }
    removed
  }

  pub(super) fn unsubscribe_parent(&self, path: &InternedPath) {
    self.directories.remove_if_mut(path, |_, watcher| {
      watcher.subscriptions.parent = None;
      watcher.is_unused()
    });
  }
}

impl ContextLookup for WatcherManager {
  fn subscriptions(&self, path: &InternedPath) -> DirectorySubscriptions {
    self
      .directories
      .get(path)
      .map_or_else(DirectorySubscriptions::default, |watcher| {
        watcher.subscriptions.clone()
      })
  }
}
