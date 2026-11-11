use std::{
  path::{Component, Path, PathBuf},
  sync::{Arc, RwLock},
};

use rspack_paths::{InternedPath, InternedPathMap, InternedPathSet};

use crate::{WatchPattern, paths::PathManager};

struct Resolution {
  target: InternedPath,
  links: InternedPathSet,
}

/// A watch cycle shares parent resolutions, including parents of missing files.
/// Unlike canonicalize, read_link does not require the final target to exist.
#[derive(Default)]
pub(crate) struct LinkResolver {
  cache: InternedPathMap<Arc<Resolution>>,
  resolving: InternedPathSet,
}

impl LinkResolver {
  fn resolve(&mut self, path: &InternedPath) -> Option<Arc<Resolution>> {
    if let Some(resolution) = self.cache.get(path) {
      return Some(Arc::clone(resolution));
    }
    if !self.resolving.insert(path.clone()) {
      return None;
    }
    let resolution = self.resolve_uncached(path);
    self.resolving.remove(path);
    if let Some(resolution) = &resolution {
      self.cache.insert(path.clone(), Arc::clone(resolution));
    }
    resolution
  }

  fn resolve_uncached(&mut self, path: &InternedPath) -> Option<Arc<Resolution>> {
    let Some(parent) = path.parent().filter(|parent| *parent != path.as_ref()) else {
      return Some(Arc::new(Resolution {
        target: path.clone(),
        links: InternedPathSet::default(),
      }));
    };
    let parent = self.resolve(&InternedPath::from(parent))?;
    let real = InternedPath::from(parent.target.join(path.file_name()?));
    let mut links = parent.links.clone();
    let target = if let Ok(content) = std::fs::read_link(&real) {
      links.insert(real);
      let target = self.resolve(&InternedPath::from(normalize(
        &parent.target.join(dunce::simplified(&content)),
      )))?;
      links.extend(target.links.iter().cloned());
      target.target.clone()
    } else {
      real
    };
    Some(Arc::new(Resolution { target, links }))
  }
}

fn normalize(path: &Path) -> PathBuf {
  let mut result = PathBuf::new();
  for component in path.components() {
    match component {
      Component::CurDir => {}
      Component::ParentDir => {
        result.pop();
      }
      component => result.push(component.as_os_str()),
    }
  }
  result
}

#[derive(Default)]
pub(crate) struct LinkSubscriptions {
  targets: InternedPathMap<InternedPathSet>,
  target_children: InternedPathMap<InternedPathSet>,
  directories: InternedPathMap<InternedPathSet>,
  resolved_directories: InternedPathMap<InternedPath>,
  directory_targets: InternedPathMap<InternedPath>,
  context_files: InternedPathSet,
  links: InternedPathMap<InternedPathSet>,
  patterns: Vec<WatchPattern>,
}

impl LinkSubscriptions {
  pub(crate) async fn add(
    &mut self,
    resolver: &mut LinkResolver,
    path: InternedPath,
    directory: bool,
    nested: bool,
    manager: &PathManager,
  ) {
    let Some(resolution) = resolver.resolve(&path) else {
      return;
    };
    if resolution.links.is_empty() {
      return;
    }
    if nested && !directory {
      // A target in the same directory is already covered by that context.
      // The context scan also records directory cycles as plain links.
      if resolution.target.is_dir()
        || path.parent().is_some_and(|parent| {
          resolver
            .resolve(&InternedPath::from(parent))
            .is_some_and(|parent| resolution.target.parent() == Some(parent.target.as_ref()))
        })
      {
        return;
      }
      self.context_files.insert(path.clone());
    }
    for link in &resolution.links {
      if !nested && manager.is_ignored_path(link).await {
        continue;
      }
      self
        .links
        .entry(link.clone())
        .or_default()
        .insert(path.clone());
      self.watch(link, false);
    }
    if !nested && manager.is_ignored_path(&resolution.target).await {
      return;
    }
    if directory && !nested {
      self
        .resolved_directories
        .insert(path.clone(), resolution.target.clone());
    }
    if directory {
      self
        .directory_targets
        .insert(path.clone(), resolution.target.clone());
    }
    self
      .targets
      .entry(resolution.target.clone())
      .or_default()
      .insert(path.clone());
    let mut child = resolution.target.clone();
    for ancestor in resolution.target.ancestors().skip(1) {
      let parent = InternedPath::from(ancestor);
      if !self
        .target_children
        .entry(parent.clone())
        .or_default()
        .insert(child)
      {
        break;
      }
      child = parent;
    }
    self.watch(&resolution.target, directory);
    // Directory aliases also translate parent removal/creation events. In
    // particular, mkdir of a missing target's parent must recover its files.
    for ancestor in path.ancestors() {
      let ancestor = InternedPath::from(ancestor);
      if let Some(resolved) = resolver.cache.get(&ancestor)
        && resolved.target != ancestor
        && (ancestor != path || directory)
      {
        self
          .directories
          .entry(resolved.target.clone())
          .or_default()
          .insert(ancestor);
      }
    }
  }

  fn watch(&mut self, path: &InternedPath, directory: bool) {
    let mut candidate = if directory {
      Some(path.as_ref())
    } else {
      path.parent()
    };
    let mut recursive = directory;
    while let Some(path) = candidate {
      if path.is_dir() {
        self.patterns.push(WatchPattern {
          path: InternedPath::from(path),
          mode: if recursive {
            notify::RecursiveMode::Recursive
          } else {
            notify::RecursiveMode::NonRecursive
          },
        });
        break;
      }
      recursive = true;
      candidate = path.parent();
    }
  }
}

/// Published as a whole so event processing never observes a half-built index.
#[derive(Default)]
pub(crate) struct Symlinks(RwLock<LinkSubscriptions>);

pub(crate) struct SymlinkAlias {
  pub path: InternedPath,
  pub link_changed: bool,
  pub resolved_path: Option<InternedPath>,
}

impl Symlinks {
  pub(crate) fn context_files(&self) -> InternedPathSet {
    self
      .0
      .read()
      .expect("symlink subscriptions lock poisoned")
      .context_files
      .clone()
  }

  pub(crate) fn replace(
    &self,
    mut subscriptions: LinkSubscriptions,
  ) -> (Vec<WatchPattern>, Vec<InternedPath>) {
    let patterns = std::mem::take(&mut subscriptions.patterns);
    let mut previous = self.0.write().expect("symlink subscriptions lock poisoned");
    let retargeted = subscriptions
      .directory_targets
      .iter()
      .filter(|(path, target)| {
        previous
          .directory_targets
          .get(*path)
          .is_some_and(|old| old != *target)
      })
      .map(|(path, _)| path.clone())
      .collect();
    *previous = subscriptions;
    (patterns, retargeted)
  }

  /// Changing a link invalidates its dependencies even when the replacement
  /// target has the same modification time.
  pub(crate) fn aliases(&self, path: &InternedPath) -> Vec<SymlinkAlias> {
    let subscriptions = self.0.read().expect("symlink subscriptions lock poisoned");
    let mut aliases = InternedPathMap::default();
    if let Some(targets) = subscriptions.targets.get(path) {
      for target in targets {
        aliases.insert(target.clone(), false);
      }
    }
    let mut pending = vec![path.clone()];
    while let Some(parent) = pending.pop() {
      if let Some(children) = subscriptions.target_children.get(&parent) {
        for child in children {
          if let Some(targets) = subscriptions.targets.get(child) {
            for target in targets {
              aliases.entry(target.clone()).or_insert(false);
            }
          }
          pending.push(child.clone());
        }
      }
    }
    for ancestor in path.ancestors() {
      let ancestor = InternedPath::from(ancestor);
      if let Some(directories) = subscriptions.directories.get(&ancestor) {
        let suffix = path.strip_prefix(&ancestor).expect("path ancestor");
        for directory in directories {
          let alias = if suffix.as_os_str().is_empty() {
            directory.clone()
          } else {
            InternedPath::from(directory.join(suffix))
          };
          aliases.entry(alias).or_insert(false);
        }
      }
    }
    if let Some(targets) = subscriptions.links.get(path) {
      for target in targets {
        aliases.insert(target.clone(), true);
      }
    }
    aliases
      .into_iter()
      .map(|(alias, link_changed)| {
        let resolved_path = if link_changed {
          None
        } else {
          alias.ancestors().skip(1).find_map(|parent| {
            subscriptions
              .resolved_directories
              .get(&InternedPath::from(parent))
              .map(|target| {
                InternedPath::from(target.join(alias.strip_prefix(parent).expect("path ancestor")))
              })
          })
        };
        SymlinkAlias {
          path: alias,
          link_changed,
          resolved_path,
        }
      })
      .collect()
  }
}
