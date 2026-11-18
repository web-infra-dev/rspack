use std::sync::RwLock;

use rspack_paths::{InternedPath, InternedPathMap};
use smallvec::SmallVec;

pub(crate) type SubscriberPaths = SmallVec<[InternedPath; 1]>;

pub(crate) struct FileSubscription {
  pub observed: InternedPath,
  pub logical: InternedPath,
}

impl From<InternedPath> for FileSubscription {
  fn from(path: InternedPath) -> Self {
    Self {
      observed: path.clone(),
      logical: path,
    }
  }
}

pub(crate) enum Subscription {
  File(InternedPath),
  Directory,
  Missing,
}

#[derive(Default)]
struct EntrySubscriptions {
  files: SubscriberPaths,
  directory: bool,
  missing: bool,
}

impl EntrySubscriptions {
  fn direct_paths(
    &self,
    path: &InternedPath,
    metadata: Option<&std::fs::Metadata>,
  ) -> SubscriberPaths {
    let file = metadata.is_none_or(std::fs::Metadata::is_file);
    let directory = metadata.is_none_or(std::fs::Metadata::is_dir);
    let mut paths = if file {
      self.files.clone()
    } else {
      SubscriberPaths::new()
    };
    if ((directory && self.directory) || ((file || directory) && self.missing))
      && !paths.contains(path)
    {
      paths.push(path.clone());
    }
    paths
  }

  fn recursive(&self) -> bool {
    self.directory || self.missing
  }

  fn is_empty(&self) -> bool {
    self.files.is_empty() && !self.recursive()
  }
}

#[derive(Default)]
struct DirectoryState {
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

impl DirectorySubscriptions {
  /// Returns whether the first subscription of this kind was added or the last removed.
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
    let recursive = entry.recursive();
    let changed = match subscription {
      Subscription::File(logical) if add => {
        let first = entry.files.is_empty();
        if !entry.files.contains(&logical) {
          entry.files.push(logical);
        }
        first
      }
      Subscription::File(logical) => {
        let Some(index) = entry.files.iter().position(|file| file == &logical) else {
          return false;
        };
        entry.files.swap_remove(index);
        entry.files.is_empty()
      }
      Subscription::Directory => std::mem::replace(&mut entry.directory, add) != add,
      Subscription::Missing => std::mem::replace(&mut entry.missing, add) != add,
    };
    if recursive != entry.recursive() {
      let mut pending = vec![path.clone()];
      while let Some(dir) = pending.pop() {
        if let Some(state) = dirs.get_mut(&dir) {
          if add {
            state.contexts.push(path.clone());
          } else {
            state.contexts.retain(|p| p != path);
          }
          pending.extend(state.entries.keys().filter(|p| **p != dir).cloned());
        }
      }
    }
    if !add {
      Self::prune(&mut dirs, path);
    }
    changed
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
      if child != parent && dirs.contains_key(&child) {
        break;
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
      if !state.entries.is_empty() {
        break;
      }
      dirs.remove(&parent);
      if child == parent {
        break;
      }
      child = parent;
    }
  }

  pub fn files(&self, path: &InternedPath) -> SubscriberPaths {
    let dirs = self
      .directories
      .read()
      .expect("directory subscriptions poisoned");
    dirs
      .get(&parent(path))
      .and_then(|dir| dir.entries.get(path))
      .map_or_default(|entry| entry.files.clone())
  }

  /// The hot path reads the parent once for both the named entry and its contexts.
  pub fn subscribers(
    &self,
    path: &InternedPath,
    metadata: Option<&std::fs::Metadata>,
  ) -> (SubscriberPaths, SubscriberPaths) {
    let dirs = self
      .directories
      .read()
      .expect("directory subscriptions poisoned");
    let dir = parent(path);
    if let Some(state) = dirs.get(&dir) {
      let direct = state
        .entries
        .get(path)
        .map_or_default(|entry| entry.direct_paths(path, metadata));
      let contexts = if dir == *path {
        SubscriberPaths::new()
      } else {
        state.contexts.clone()
      };
      return (direct, contexts);
    }
    // An event may name an as-yet unregistered directory inside a context.
    for child in dir.ancestors().map(InternedPath::from) {
      if let Some(state) = dirs.get(&parent(&child)) {
        let mut contexts = state.contexts.clone();
        if state
          .entries
          .get(&child)
          .is_some_and(EntrySubscriptions::recursive)
        {
          contexts.push(child);
        }
        return (SubscriberPaths::new(), contexts);
      }
    }
    Default::default()
  }

  pub fn children(&self, path: &InternedPath) -> Vec<InternedPath> {
    self
      .directories
      .read()
      .expect("directory subscriptions poisoned")
      .get(path)
      .map_or_default(|dir| {
        dir
          .entries
          .keys()
          .filter(|child| *child != path)
          .cloned()
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
