use std::sync::{Arc, Mutex, Weak};

use rspack_paths::{InternedPath, InternedPathDashMap, InternedPathMap};

use crate::{
  FsEventKind,
  observation::{Observations, Scan},
};

pub(super) enum MemberKind {
  File,
  Directory,
}

// An initial scan can follow a directory symlink that a live event records as a file.
#[derive(Default)]
pub(super) struct ContextMember {
  pub(super) file: bool,
  pub(super) directory: bool,
  pending: Weak<PendingContextChange>,
}

pub(super) struct PendingContextChange {
  subscribers: Mutex<Vec<Weak<InternedPath>>>,
}

impl PendingContextChange {
  fn merge(slot: &mut Weak<Self>, subscribers: Vec<Weak<InternedPath>>) -> Arc<Self> {
    if let Some(pending) = slot.upgrade() {
      let mut owed = pending.subscribers.lock().expect("context change poisoned");
      owed.retain(|subscriber| subscriber.strong_count() != 0);
      if !owed.is_empty() {
        for subscriber in subscribers {
          if !owed.iter().any(|old| old.ptr_eq(&subscriber)) {
            owed.push(subscriber);
          }
        }
        return Arc::clone(&pending);
      }
    }
    let pending = Arc::new(Self {
      subscribers: Mutex::new(subscribers),
    });
    *slot = Arc::downgrade(&pending);
    pending
  }

  pub(super) fn has_subscribers(&self) -> bool {
    self
      .subscribers
      .lock()
      .expect("context change poisoned")
      .iter()
      .any(|s| s.strong_count() != 0)
  }

  pub(super) fn take_subscribers(&self) -> Vec<InternedPath> {
    std::mem::take(&mut *self.subscribers.lock().expect("context change poisoned"))
      .into_iter()
      .filter_map(|s| s.upgrade().map(|path| (*path).clone()))
      .collect()
  }

  pub(super) fn acknowledge(&self, delivered: &[(InternedPath, FsEventKind)]) {
    self
      .subscribers
      .lock()
      .expect("context change poisoned")
      .retain(|s| {
        s.upgrade()
          .is_some_and(|path| !delivered.iter().any(|(dep, _)| dep == &*path))
      });
  }
}

pub(super) struct ContextChange {
  pub(super) scan: Scan,
  pub(super) directory: bool,
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
pub(super) struct DirectoryWatcher {
  pub(super) members: InternedPathMap<ContextMember>,
  // Each registration owns a distinct handle, including a re-registration of the same path.
  explicit: Option<Arc<InternedPath>>,
  parent: Option<InternedPath>,
}

impl DirectoryWatcher {
  fn is_unused(&self) -> bool {
    self.members.is_empty() && self.explicit.is_none() && self.parent.is_none()
  }
}

#[derive(Default)]
pub(super) struct WatcherManager {
  pub(super) directories: InternedPathDashMap<DirectoryWatcher>,
}

impl WatcherManager {
  pub(super) fn subscribe_context(&self, path: &InternedPath) {
    let mut watcher = self.directories.entry(path.clone()).or_default();
    watcher.explicit = Some(Arc::new(path.clone()));
  }

  pub(super) fn unsubscribe_context(&self, path: &InternedPath) {
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

  pub(super) fn queue_context_change(
    &self,
    path: &InternedPath,
    directory: bool,
    scan: &Scan,
    state: &Observations,
    missing_subscriber: impl Fn(&InternedPath) -> Option<Weak<InternedPath>>,
  ) -> Option<ContextChange> {
    let mut subscribers = Vec::new();
    for ancestor in path.ancestors().skip(usize::from(!directory)) {
      let dep = InternedPath::from(ancestor);
      if let Some(watcher) = self.directories.get(&dep)
        && let Some(subscription) = &watcher.explicit
        && state.same_subscription(scan, &dep)
      {
        subscribers.push(Arc::downgrade(subscription));
      }
      subscribers.extend(missing_subscriber(&dep));
    }
    if subscribers.is_empty() {
      return None;
    }
    let parent = InternedPath::from(path.parent()?);
    let mut watcher = self.directories.get_mut(&parent)?;
    let member = watcher.members.get_mut(path)?;
    let pending = PendingContextChange::merge(&mut member.pending, subscribers);
    Some(ContextChange {
      scan: scan.clone(),
      directory,
      pending,
    })
  }

  pub(super) fn pending_change(&self, path: &InternedPath) -> Option<Arc<PendingContextChange>> {
    let parent = InternedPath::from(path.parent()?);
    self
      .directories
      .get(&parent)?
      .members
      .get(path)?
      .pending
      .upgrade()
  }

  pub(super) fn is_watched_directory(&self, path: &InternedPath) -> bool {
    self
      .directories
      .get(path)
      .is_some_and(|watcher| watcher.explicit.is_some() || watcher.parent.is_some())
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
