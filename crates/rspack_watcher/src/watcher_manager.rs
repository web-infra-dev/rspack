use std::sync::{Arc, Mutex, MutexGuard};

use rspack_paths::{InternedPath, InternedPathDashMap, InternedPathMap};

use crate::{
  observation::{Mutation, Observations},
  paths::{DirectoryScan, FileRecord},
};

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
  pub(super) fn is_active(&self) -> bool {
    self.explicit || self.parent.is_some()
  }
}

pub(super) trait ContextLookup {
  fn subscriptions(&self, path: &InternedPath) -> DirectorySubscriptions;
}

#[derive(Default)]
pub(super) struct DirectoryWatcher {
  scan: Option<Arc<DirectoryScan>>,
  registered_children: usize,
  pub(super) members: InternedPathMap<ContextMember>,
  pub(super) files: InternedPathMap<FileRecord>,
  pub(super) last_watch_event: Option<u64>,
  pub(super) subscriptions: DirectorySubscriptions,
}

impl DirectoryWatcher {
  pub(super) fn owner(path: &InternedPath) -> InternedPath {
    path
      .parent()
      .map_or_else(|| path.clone(), InternedPath::from)
  }

  pub(super) fn is_unused(&self) -> bool {
    self.registered_children == 0
      && self.members.is_empty()
      && self.files.is_empty()
      && self.last_watch_event.is_none()
      && !self.subscriptions.is_active()
  }

  pub(super) fn clear_file(&mut self, path: &InternedPath, clear: impl FnOnce(&mut FileRecord)) {
    if let Some(record) = self.files.get_mut(path) {
      clear(record);
      if record.is_empty() {
        self.files.remove(path);
      }
    }
  }
}

#[derive(Default)]
pub(super) struct WatcherManager {
  pub(super) directories: Arc<InternedPathDashMap<DirectoryWatcher>>,
  pub(super) observations: Arc<Mutex<Observations>>,
}

impl WatcherManager {
  pub(super) fn directory_scan(&self, dir: &InternedPath) -> Option<Arc<DirectoryScan>> {
    let mut watcher = self.directories.get_mut(dir)?;
    Some(Arc::clone(watcher.scan.get_or_insert_with(|| {
      Arc::new(DirectoryScan::new(dir.clone()))
    })))
  }

  pub(super) fn register_child(&self, path: &InternedPath) {
    let owner = DirectoryWatcher::owner(path);
    let mut watcher = self.directories.entry(owner).or_default();
    watcher.registered_children += 1;
  }

  pub(super) fn unregister_child(&self, path: &InternedPath) {
    let owner = DirectoryWatcher::owner(path);
    self.directories.remove_if_mut(&owner, |_, watcher| {
      watcher.registered_children -= 1;
      watcher.is_unused()
    });
  }

  pub(super) fn clear_event_time(&self, path: &InternedPath) {
    self.directories.remove_if_mut(path, |_, watcher| {
      watcher.last_watch_event = None;
      watcher.is_unused()
    });
  }

  pub(super) fn event_time(&self, path: &InternedPath) -> Option<u64> {
    self.directories.get(path)?.last_watch_event
  }

  pub(super) fn lock_observations(&self) -> MutexGuard<'_, Observations> {
    self
      .observations
      .lock()
      .expect("watch observations poisoned")
  }

  pub(super) fn begin_metadata_update(&self, path: &InternedPath) -> MutexGuard<'_, Observations> {
    let mut state = self.lock_observations();
    state.observe_live(path, Mutation::Metadata);
    state
  }

  pub(super) fn subscribe_context(&self, path: &InternedPath) {
    let mut watcher = self.directories.entry(path.clone()).or_default();
    watcher.subscriptions.explicit = true;
  }

  pub(super) fn unsubscribe_context(&self, path: &InternedPath) {
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
