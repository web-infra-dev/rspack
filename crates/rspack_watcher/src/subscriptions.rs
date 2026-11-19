use std::sync::RwLock;

use rspack_paths::{InternedPath, InternedPathMap};
use smallvec::SmallVec;

pub(crate) type SubscriberPaths = SmallVec<[InternedPath; 1]>;

pub(crate) struct PathSubscription {
  pub watched_path: InternedPath,
  pub requested_path: InternedPath,
}

impl From<InternedPath> for PathSubscription {
  fn from(path: InternedPath) -> Self {
    Self {
      watched_path: path.clone(),
      requested_path: path,
    }
  }
}

pub(crate) enum Subscription {
  File(InternedPath),
  Directory(InternedPath),
  Missing(InternedPath),
}

#[derive(Default)]
struct EntrySubscriptions {
  files: SubscriberPaths,
  directories: SubscriberPaths,
  missing: SubscriberPaths,
}

pub(crate) struct Subscriber {
  pub path: InternedPath,
  pub file_or_missing: bool,
}

pub(crate) type Subscribers = SmallVec<[Subscriber; 1]>;

pub(crate) struct ContextSubscribers {
  pub watched: InternedPath,
  pub requested: Subscribers,
}

impl EntrySubscriptions {
  fn subscribers(&self, file: bool, directory: bool) -> Subscribers {
    let mut requested = Subscribers::new();
    let mut add = |path: &InternedPath, file_or_missing| {
      if !requested.iter().any(|subscriber| subscriber.path == *path) {
        requested.push(Subscriber {
          path: path.clone(),
          file_or_missing,
        });
      }
    };
    if file {
      for path in &self.files {
        add(path, true);
      }
    }
    if directory || file {
      for path in &self.missing {
        add(path, true);
      }
    }
    if directory {
      for path in &self.directories {
        add(path, self.files.contains(path));
      }
    }
    requested
  }

  fn recursive(&self) -> bool {
    !self.directories.is_empty() || !self.missing.is_empty()
  }

  fn is_empty(&self) -> bool {
    self.files.is_empty() && !self.recursive()
  }
}

#[derive(Default)]
struct DirectoryState {
  scanned: bool,
  /// Number of explicitly registered entries below this directory.
  registered_entries: usize,
  entries: InternedPathMap<EntrySubscriptions>,
  /// Contexts (including registered missing directories) that receive child events.
  contexts: SubscriberPaths,
}

#[derive(Default)]
pub(crate) struct DirectorySubscriptions {
  directories: RwLock<InternedPathMap<DirectoryState>>,
}

fn parent(path: &InternedPath) -> InternedPath {
  path
    .parent()
    .map_or_else(|| path.clone(), InternedPath::from)
}

fn update_paths(paths: &mut SubscriberPaths, requested: InternedPath, add: bool) -> bool {
  if add {
    if paths.contains(&requested) {
      return false;
    }
    paths.push(requested);
    true
  } else if let Some(index) = paths.iter().position(|p| p == &requested) {
    paths.swap_remove(index);
    paths.is_empty()
  } else {
    false
  }
}

impl DirectorySubscriptions {
  /// A new request needs an initial scan. Release the watched path after the last request.
  pub fn update(&self, path: &InternedPath, subscription: Subscription, add: bool) -> bool {
    let mut dirs = self
      .directories
      .write()
      .expect("directory subscriptions poisoned");
    let dir = parent(path);
    if add {
      Self::ensure_directory(&mut dirs, &dir);
    }
    let Some(state) = dirs.get_mut(&dir) else {
      return false;
    };
    let entry = if add {
      state.entries.entry(path.clone()).or_default()
    } else if let Some(entry) = state.entries.get_mut(path) {
      entry
    } else {
      return false;
    };
    let was_empty = entry.is_empty();
    let recursive = entry.recursive();
    let changed = match subscription {
      Subscription::File(requested) => update_paths(&mut entry.files, requested, add),
      Subscription::Missing(requested) => update_paths(&mut entry.missing, requested, add),
      Subscription::Directory(requested) => update_paths(&mut entry.directories, requested, add),
    };
    let recursive_changed = recursive != entry.recursive();
    if was_empty != entry.is_empty() {
      for ancestor in dir.ancestors().map(InternedPath::from) {
        let state = dirs.get_mut(&ancestor).expect("registered ancestor");
        if add {
          state.registered_entries += 1;
        } else {
          state.registered_entries -= 1;
        }
      }
    }
    if recursive_changed {
      Self::update_subtree(&mut dirs, path, |state| {
        if add {
          state.contexts.push(path.clone());
        } else {
          state.contexts.retain(|p| p != path);
          state.scanned &= !state.contexts.is_empty();
        }
      });
    }
    if !add {
      Self::prune(&mut dirs, path);
    }
    changed
  }

  fn update_subtree(
    dirs: &mut InternedPathMap<DirectoryState>,
    root: &InternedPath,
    mut update: impl FnMut(&mut DirectoryState),
  ) {
    let mut pending = vec![root.clone()];
    let mut released = Vec::new();
    while let Some(dir) = pending.pop() {
      if let Some(state) = dirs.get_mut(&dir) {
        let scanned = state.scanned;
        update(state);
        if scanned && !state.scanned {
          released.push(dir.clone());
        }
        pending.extend(state.entries.keys().filter(|p| **p != dir).cloned());
      }
    }
    for dir in released.into_iter().rev() {
      Self::prune(dirs, &dir);
    }
  }

  fn ensure_directory(dirs: &mut InternedPathMap<DirectoryState>, path: &InternedPath) {
    if dirs.contains_key(path) {
      return;
    }
    let mut contexts = SubscriberPaths::new();
    if let Some(parent) = path.parent().map(InternedPath::from) {
      Self::ensure_directory(dirs, &parent);
      let state = dirs.get_mut(&parent).expect("registered parent");
      contexts.clone_from(&state.contexts);
      if state.entries.entry(path.clone()).or_default().recursive() {
        contexts.push(path.clone());
      }
    }
    dirs.insert(
      path.clone(),
      DirectoryState {
        contexts,
        ..Default::default()
      },
    );
  }

  fn prune(dirs: &mut InternedPathMap<DirectoryState>, path: &InternedPath) {
    let mut child = path.clone();
    loop {
      let parent = parent(&child);
      if child != parent
        && let Some(state) = dirs.get(&child)
      {
        if state.scanned || !state.entries.is_empty() {
          break;
        }
        dirs.remove(&child);
      }
      let Some(state) = dirs.get_mut(&parent) else {
        break;
      };
      if state
        .entries
        .get(&child)
        .is_some_and(EntrySubscriptions::is_empty)
      {
        state.entries.remove(&child);
      }
      if state.scanned || !state.entries.is_empty() {
        break;
      }
      dirs.remove(&parent);
      if child == parent {
        break;
      }
      child = parent;
    }
  }

  pub fn scan_directory(&self, path: &InternedPath) {
    let mut dirs = self
      .directories
      .write()
      .expect("directory subscriptions poisoned");
    Self::ensure_directory(&mut dirs, path);
    let state = dirs.get_mut(path).expect("directory just inserted");
    if state.contexts.is_empty() {
      Self::prune(&mut dirs, path);
    } else {
      state.scanned = true;
    }
  }

  pub fn forget_scanned_subtree(&self, path: &InternedPath) {
    let mut dirs = self
      .directories
      .write()
      .expect("directory subscriptions poisoned");
    Self::update_subtree(&mut dirs, path, |state| state.scanned = false);
  }

  pub fn files(&self, path: &InternedPath) -> SubscriberPaths {
    self.entry_paths(path, |entry| entry.files.clone())
  }

  pub fn missing(&self, path: &InternedPath) -> SubscriberPaths {
    self.entry_paths(path, |entry| entry.missing.clone())
  }

  pub fn directories(&self, path: &InternedPath) -> SubscriberPaths {
    self.entry_paths(path, |entry| entry.directories.clone())
  }

  fn entry_paths(
    &self,
    path: &InternedPath,
    select: impl FnOnce(&EntrySubscriptions) -> SubscriberPaths,
  ) -> SubscriberPaths {
    let dirs = self
      .directories
      .read()
      .expect("directory subscriptions poisoned");
    dirs
      .get(&parent(path))
      .and_then(|dir| dir.entries.get(path))
      .map_or_default(select)
  }

  pub fn subscribers(
    &self,
    path: &InternedPath,
    metadata: Option<&std::fs::Metadata>,
  ) -> (Subscribers, SmallVec<[ContextSubscribers; 1]>) {
    let dirs = self
      .directories
      .read()
      .expect("directory subscriptions poisoned");
    let (entry, contexts) = Self::entry_and_contexts(&dirs, path);
    let direct = entry.map_or_default(|entry| {
      entry.subscribers(
        metadata.is_none_or(std::fs::Metadata::is_file),
        metadata.is_none_or(std::fs::Metadata::is_dir),
      )
    });
    let contexts = contexts
      .map(|(watched, entry)| ContextSubscribers {
        watched: watched.clone(),
        requested: entry.subscribers(false, true),
      })
      .collect();
    (direct, contexts)
  }

  pub fn is_below_context(&self, path: &InternedPath) -> bool {
    let dirs = self
      .directories
      .read()
      .expect("directory subscriptions poisoned");
    let (_, mut contexts) = Self::entry_and_contexts(&dirs, path);
    contexts.any(|(_, entry)| !entry.directories.is_empty())
  }

  fn entry_and_contexts<'a>(
    dirs: &'a InternedPathMap<DirectoryState>,
    path: &InternedPath,
  ) -> (
    Option<&'a EntrySubscriptions>,
    impl Iterator<Item = (&'a InternedPath, &'a EntrySubscriptions)> + 'a,
  ) {
    let mut child = path.clone();
    let (direct, contexts, child_context) = loop {
      let dir = parent(&child);
      if let Some(state) = dirs.get(&dir) {
        let entry = state.entries.get_key_value(&child);
        let contexts: &[_] = if dir == *path { &[] } else { &state.contexts };
        if child == *path {
          break (entry.map(|(_, entry)| entry), contexts, None);
        }
        // A hint below an unscanned directory belongs to the nearest known
        // parent's contexts and to the named child if it has recursive subscribers.
        break (None, contexts, entry.filter(|(_, entry)| entry.recursive()));
      }
      if dir == child {
        break (None, &[][..], None);
      }
      child = dir;
    };
    let ancestors = contexts.iter().filter_map(|context| {
      let entry = dirs.get(&parent(context))?.entries.get(context)?;
      Some((context, entry))
    });
    (direct, ancestors.chain(child_context))
  }

  pub fn registered_children(&self, path: &InternedPath) -> Vec<InternedPath> {
    let dirs = self
      .directories
      .read()
      .expect("directory subscriptions poisoned");
    dirs
      .get(path)
      .filter(|dir| dir.registered_entries != 0)
      .map_or_default(|dir| {
        dir
          .entries
          .iter()
          .filter(|(child, entry)| {
            *child != path
              && (!entry.is_empty()
                || dirs
                  .get(*child)
                  .is_some_and(|state| state.registered_entries != 0))
          })
          .map(|(child, _)| child.clone())
          .collect()
      })
  }

  pub fn contains_directory(&self, path: &InternedPath) -> bool {
    self
      .directories
      .read()
      .expect("directory subscriptions poisoned")
      .contains_key(path)
  }

  pub fn directory_count(&self) -> usize {
    self
      .directories
      .read()
      .expect("directory subscriptions poisoned")
      .len()
  }
}
