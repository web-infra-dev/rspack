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
    let recursive = entry.recursive();
    let changed = match subscription {
      Subscription::File(requested) => update_paths(&mut entry.files, requested, add),
      Subscription::Missing(requested) => update_paths(&mut entry.missing, requested, add),
      Subscription::Directory(requested) => update_paths(&mut entry.directories, requested, add),
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

  /// The hot path reads the parent once for both the named entry and its contexts.
  pub fn subscribers(
    &self,
    path: &InternedPath,
    metadata: Option<&std::fs::Metadata>,
  ) -> (Subscribers, SmallVec<[ContextSubscribers; 1]>) {
    let dirs = self
      .directories
      .read()
      .expect("directory subscriptions poisoned");
    let context_subscribers = |contexts: &SubscriberPaths| {
      contexts
        .iter()
        .filter_map(|watched| {
          let entry = dirs.get(&parent(watched))?.entries.get(watched)?;
          Some(ContextSubscribers {
            watched: watched.clone(),
            requested: entry.subscribers(false, true),
          })
        })
        .collect()
    };
    let dir = parent(path);
    if let Some(state) = dirs.get(&dir) {
      let direct = state.entries.get(path).map_or_default(|entry| {
        entry.subscribers(
          metadata.is_none_or(std::fs::Metadata::is_file),
          metadata.is_none_or(std::fs::Metadata::is_dir),
        )
      });
      let contexts = if dir == *path {
        SmallVec::new()
      } else {
        context_subscribers(&state.contexts)
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
        return (Subscribers::new(), context_subscribers(&contexts));
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
