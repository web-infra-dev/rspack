use std::{sync::Arc, time::SystemTime};

use rspack_paths::{InternedPath, InternedPathDashMap};

use crate::directory_watcher::{ContextMember, DirectoryWatcher, FileObservation};

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

  pub(super) fn set_directory(&self, path: &InternedPath) -> bool {
    let parent = DirectoryWatcher::owner(path);
    let added = self
      .directories
      .entry(parent.clone())
      .or_default()
      .set_directory(path);
    self
      .directories
      .entry(path.clone())
      .or_default()
      .subscriptions
      .parent = Some(parent);
    added
  }

  pub(super) fn set_file_time(
    &self,
    path: &InternedPath,
    mtime: Option<SystemTime>,
    origin: FileObservation,
  ) -> bool {
    self
      .directories
      .entry(DirectoryWatcher::owner(path))
      .or_default()
      .set_file_time(path, mtime, origin)
  }

  pub(super) fn has_member(
    &self,
    path: &InternedPath,
    matches: impl FnOnce(&ContextMember) -> bool,
  ) -> bool {
    path.parent().is_some_and(|parent| {
      self
        .directories
        .get(&InternedPath::from(parent))
        .is_some_and(|watcher| watcher.members.get(path).is_some_and(matches))
    })
  }

  pub(super) fn set_missing(&self, path: &InternedPath, keep_file: bool) -> ContextMember {
    let mut removed = ContextMember::default();
    self
      .directories
      .remove_if_mut(&DirectoryWatcher::owner(path), |_, watcher| {
        removed = watcher.set_missing(path, keep_file);
        watcher.is_unused()
      });
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
