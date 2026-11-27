use std::sync::{Arc, Mutex, MutexGuard, Weak};

use rspack_paths::{InternedPath, InternedPathDashMap, InternedPathMap};

use crate::{
  observation::{Mutation, Observations},
  paths::FileRecord,
};

type PendingChanges = InternedPathDashMap<Weak<PendingContextChange>>;

pub(super) struct PendingContextChange {
  pub(super) path: InternedPath,
  registry: Weak<PendingChanges>,
  subscribers: Mutex<Vec<Weak<InternedPath>>>,
}

impl PendingContextChange {
  pub(super) fn register(
    registry: &Arc<PendingChanges>,
    path: &InternedPath,
    subscribers: Vec<Weak<InternedPath>>,
  ) -> Option<Arc<Self>> {
    if subscribers.is_empty() {
      return None;
    }
    if let Some(pending) = registry.get(path).and_then(|slot| slot.upgrade()) {
      let mut owed = pending.subscribers.lock().expect("context change poisoned");
      owed.retain(|subscriber| subscriber.strong_count() != 0);
      if !owed.is_empty() {
        for subscriber in subscribers {
          if !owed.iter().any(|old| old.ptr_eq(&subscriber)) {
            owed.push(subscriber);
          }
        }
        return Some(Arc::clone(&pending));
      }
    }
    let change = Arc::new(Self {
      path: path.clone(),
      registry: Arc::downgrade(registry),
      subscribers: Mutex::new(subscribers),
    });
    registry.insert(path.clone(), Arc::downgrade(&change));
    Some(change)
  }

  pub(super) fn has_subscribers(&self) -> bool {
    self
      .subscribers
      .lock()
      .expect("context change poisoned")
      .iter()
      .any(|subscriber| subscriber.strong_count() != 0)
  }

  pub(super) fn take_subscribers(&self) -> Vec<Arc<InternedPath>> {
    std::mem::take(&mut *self.subscribers.lock().expect("context change poisoned"))
      .into_iter()
      .filter_map(|subscriber| subscriber.upgrade())
      .collect()
  }
}

impl Drop for PendingContextChange {
  fn drop(&mut self) {
    if let Some(registry) = self.registry.upgrade() {
      registry.remove_if(&self.path, |_, current| {
        std::ptr::eq(current.as_ptr(), self)
      });
    }
  }
}

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

pub(super) trait ContextLookup {
  fn subscriptions(&self, path: &InternedPath) -> DirectorySubscriptions;
}

#[derive(Default)]
pub(super) struct DirectoryWatcher {
  pub(super) members: InternedPathMap<ContextMember>,
  pub(super) files: InternedPathMap<FileRecord>,
  pub(super) last_watch_event: Option<u64>,
  explicit: Option<Arc<InternedPath>>,
  parent: Option<InternedPath>,
}

impl DirectoryWatcher {
  pub(super) fn owner(path: &InternedPath) -> InternedPath {
    path
      .parent()
      .map_or_else(|| path.clone(), InternedPath::from)
  }

  pub(super) fn is_watched(&self) -> bool {
    self.explicit.is_some() || self.parent.is_some()
  }

  pub(super) fn is_unused(&self) -> bool {
    self.members.is_empty()
      && self.files.is_empty()
      && self.last_watch_event.is_none()
      && !self.is_watched()
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
  pub(super) pending_changes: Arc<PendingChanges>,
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

  pub(super) fn lock_observations(&self) -> MutexGuard<'_, Observations> {
    self
      .observations
      .lock()
      .expect("watch observations poisoned")
  }

  pub(super) fn begin_metadata_update(&self, path: &InternedPath) -> MutexGuard<'_, Observations> {
    let mut state = self.lock_observations();
    state.invalidate(path, Mutation::Metadata, None);
    state
  }

  pub(super) fn subscribe_context(&self, path: &InternedPath, state: &mut Observations) {
    state.context_changed(path);
    let mut watcher = self.directories.entry(path.clone()).or_default();
    watcher.explicit = Some(Arc::new(path.clone()));
  }

  pub(super) fn unsubscribe_context(&self, path: &InternedPath, state: &mut Observations) {
    state.context_changed(path);
    self.directories.remove_if_mut(path, |_, watcher| {
      watcher.explicit = None;
      watcher.is_unused()
    });
  }

  pub(super) fn context_subscriber(&self, path: &InternedPath) -> Option<Weak<InternedPath>> {
    self
      .directories
      .get(path)?
      .explicit
      .as_ref()
      .map(Arc::downgrade)
  }

  pub(super) fn is_explicit_context(&self, path: &InternedPath) -> bool {
    self
      .directories
      .get(path)
      .is_some_and(|watcher| watcher.explicit.is_some())
  }

  pub(super) fn is_watched_directory(&self, path: &InternedPath) -> bool {
    self
      .directories
      .get(path)
      .is_some_and(|watcher| watcher.is_watched())
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
        child.parent = Some(parent);
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
      watcher.parent = None;
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
        DirectorySubscriptions {
          explicit: watcher.explicit.is_some(),
          parent: watcher.parent.clone(),
        }
      })
  }
}
