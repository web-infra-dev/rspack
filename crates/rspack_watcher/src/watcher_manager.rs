use std::sync::{Arc, Mutex, Weak};

use rspack_paths::{InternedPath, InternedPathDashMap, InternedPathMap, InternedPathSet};

use crate::paths::{DirectoryScan, FileRecord};

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
  pub(crate) scan: Option<Arc<DirectoryScan>>,
  pub(crate) live_during_scan: Weak<Mutex<InternedPathSet>>,
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
    self.members.is_empty()
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
}

impl WatcherManager {
  pub(super) fn clear_event_time(&self, path: &InternedPath) {
    self.directories.remove_if_mut(path, |_, watcher| {
      watcher.last_watch_event = None;
      watcher.is_unused()
    });
  }

  pub(super) fn event_time(&self, path: &InternedPath) -> Option<u64> {
    self.directories.get(path)?.last_watch_event
  }

  pub(super) fn subscribe_context(&self, path: &InternedPath) {
    let mut watcher = self.directories.entry(path.clone()).or_default();
    watcher.invalidate_scan();
    watcher.subscriptions.explicit = true;
  }

  pub(super) fn unsubscribe_context(&self, path: &InternedPath) {
    self.directories.remove_if_mut(path, |_, watcher| {
      watcher.invalidate_scan();
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
    watcher.note_member(path);
    match kind {
      MemberKind::File => !std::mem::replace(
        &mut watcher.members.entry(path.clone()).or_default().file,
        true,
      ),
      MemberKind::Directory => {
        let added = watcher.set_directory(path);
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
          watcher.note_member(path);
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
      watcher.invalidate_scan();
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
