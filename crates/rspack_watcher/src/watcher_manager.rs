use std::{
  sync::{Arc, Mutex, MutexGuard, Weak},
  time::SystemTime,
};

use rspack_paths::{InternedPath, InternedPathDashMap, InternedPathSet};

use crate::{
  FsEventKind,
  directory_watcher::{ContextMember, DirectoryWatcher, FileObservation},
  observation::{Observations, Scan},
  paths::DirectoryScan,
};

#[derive(Clone, Copy, Default)]
pub(super) enum ScannedEntry {
  #[default]
  File,
  Directory(Option<SystemTime>),
  Missing {
    directory: bool,
  },
}

impl ScannedEntry {
  pub(super) fn directory(self) -> bool {
    matches!(self, Self::Directory(_) | Self::Missing { directory: true })
  }

  pub(super) fn kind(self) -> FsEventKind {
    if matches!(self, Self::Missing { .. }) {
      FsEventKind::Remove
    } else {
      FsEventKind::Change
    }
  }
}

type PendingChanges = InternedPathDashMap<Weak<PendingContextChange>>;

pub(super) struct PendingContextChange {
  path: InternedPath,
  registry: Weak<PendingChanges>,
  subscribers: Mutex<PendingSubscribers>,
}

#[derive(Default)]
struct PendingSubscribers {
  registrations: Vec<Weak<InternedPath>>,
  // The mtime observed for the directory's own missing dependency, if any.
  restored_mtime: Option<SystemTime>,
  entry: ScannedEntry,
}

impl PendingContextChange {
  fn register(
    registry: &Arc<PendingChanges>,
    path: &InternedPath,
    subscribers: PendingSubscribers,
  ) -> Arc<Self> {
    let previous = registry.get(path).and_then(|slot| slot.upgrade());
    if let Some(pending) = previous {
      let mut owed = pending.subscribers.lock().expect("context change poisoned");
      owed.registrations.retain(|s| s.strong_count() != 0);
      if !owed.registrations.is_empty() {
        owed.restored_mtime = subscribers.restored_mtime;
        owed.entry = subscribers.entry;
        for subscriber in subscribers.registrations {
          if !owed.registrations.iter().any(|old| old.ptr_eq(&subscriber)) {
            owed.registrations.push(subscriber);
          }
        }
        return Arc::clone(&pending);
      }
    }
    let pending = Arc::new(Self {
      path: path.clone(),
      registry: Arc::downgrade(registry),
      subscribers: Mutex::new(subscribers),
    });
    registry.insert(path.clone(), Arc::downgrade(&pending));
    pending
  }

  #[cfg(feature = "testing")]
  pub(super) fn has_subscribers(&self) -> bool {
    self
      .subscribers
      .lock()
      .expect("context change poisoned")
      .registrations
      .iter()
      .any(|s| s.strong_count() != 0)
  }

  pub(super) fn rechecked(&self, entry: ScannedEntry) {
    self
      .subscribers
      .lock()
      .expect("context change poisoned")
      .entry = entry;
  }

  pub(super) fn take_subscribers(
    &self,
  ) -> (Vec<Arc<InternedPath>>, Option<SystemTime>, ScannedEntry) {
    let owed = std::mem::take(&mut *self.subscribers.lock().expect("context change poisoned"));
    let subscribers = owed
      .registrations
      .into_iter()
      .filter_map(|s| s.upgrade())
      .collect();
    (subscribers, owed.restored_mtime, owed.entry)
  }

  pub(super) fn acknowledge(&self, delivered: &[(InternedPath, FsEventKind)]) {
    self
      .subscribers
      .lock()
      .expect("context change poisoned")
      .registrations
      .retain(|s| {
        s.upgrade()
          .is_some_and(|path| !delivered.iter().any(|(dep, _)| dep == &*path))
      });
  }
}

impl Drop for PendingContextChange {
  fn drop(&mut self) {
    if let Some(registry) = self.registry.upgrade() {
      registry.remove_if(&self.path, |_, pending| {
        std::ptr::eq(pending.as_ptr(), self)
      });
    }
  }
}

#[derive(Clone)]
pub(super) struct ContextChange {
  pub(super) entry: ScannedEntry,
  pub(super) pending: Arc<PendingContextChange>,
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
pub(super) struct WatcherManager {
  pub(super) directories: Arc<InternedPathDashMap<DirectoryWatcher>>,
  pub(super) observations: Arc<Mutex<Observations>>,
  pending_changes: Arc<PendingChanges>,
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
    self
      .directories
      .entry(owner)
      .or_default()
      .registered_children += 1;
  }

  pub(super) fn unregister_child(&self, path: &InternedPath) {
    self
      .directories
      .remove_if_mut(&DirectoryWatcher::owner(path), |_, watcher| {
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

  pub(super) fn member_paths(&self, dir: &InternedPath) -> InternedPathSet {
    self
      .directories
      .get(dir)
      .map_or_else(InternedPathSet::default, |w| {
        w.members.keys().cloned().collect()
      })
  }

  pub(super) fn lock_observations(&self) -> MutexGuard<'_, Observations> {
    self
      .observations
      .lock()
      .expect("watch observations poisoned")
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

  pub(super) fn is_explicit_context(&self, path: &InternedPath) -> bool {
    self
      .directories
      .get(path)
      .is_some_and(|watcher| watcher.explicit.is_some())
  }

  pub(super) fn is_context_subscriber(&self, subscriber: &Arc<InternedPath>) -> bool {
    self
      .directories
      .get(subscriber.as_ref())
      .is_some_and(|watcher| {
        watcher
          .explicit
          .as_ref()
          .is_some_and(|current| Arc::ptr_eq(subscriber, current))
      })
  }

  pub(super) fn queue_context_change(
    &self,
    path: &InternedPath,
    entry: ScannedEntry,
    scan: &Scan,
    state: &Observations,
    subscribers_for: impl Fn(&InternedPath) -> [Option<Weak<InternedPath>>; 2],
  ) -> Option<ContextChange> {
    let mut subscribers = PendingSubscribers {
      entry,
      ..Default::default()
    };
    for ancestor in path.ancestors().skip(usize::from(
      !entry.directory() && entry.kind() != FsEventKind::Remove,
    )) {
      let dep = InternedPath::from(ancestor);
      if let Some(watcher) = self.directories.get(&dep)
        && let Some(subscription) = &watcher.explicit
        && state.same_subscription(scan, &dep)
      {
        subscribers.registrations.push(Arc::downgrade(subscription));
      }
      for subscription in subscribers_for(&dep).into_iter().flatten() {
        if &dep == path
          && let ScannedEntry::Directory(mtime) = entry
        {
          subscribers.restored_mtime = mtime;
        }
        subscribers.registrations.push(subscription);
      }
    }
    if subscribers.registrations.is_empty() {
      return None;
    }
    let pending = PendingContextChange::register(&self.pending_changes, path, subscribers);
    Some(ContextChange { entry, pending })
  }

  pub(super) fn pending_change(&self, path: &InternedPath) -> Option<Arc<PendingContextChange>> {
    self.pending_changes.get(path)?.upgrade()
  }

  pub(super) fn is_watched_directory(&self, path: &InternedPath) -> bool {
    self
      .directories
      .get(path)
      .is_some_and(|watcher| watcher.explicit.is_some() || watcher.parent.is_some())
  }

  pub(super) fn set_directory(&self, path: &InternedPath) -> bool {
    let parent = DirectoryWatcher::owner(path);
    let added = self
      .directories
      .entry(parent.clone())
      .or_default()
      .set_directory(path);
    self.directories.entry(path.clone()).or_default().parent = Some(parent);
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
