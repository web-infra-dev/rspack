use std::sync::RwLock;

use rspack_paths::{InternedPath, InternedPathMap, InternedPathSet};

use crate::{
  WatchPattern,
  symlinks::{LinkResolver, SymlinkUpdate},
};

#[derive(Default)]
struct RootPaths {
  targets: InternedPathMap<InternedPath>,
  aliases: InternedPathMap<Vec<InternedPath>>,
  links: InternedPathSet,
}

/// Explicit directory roots are entered even when discovered links are not followed.
/// FSEvents reports physical paths; subscriptions and ignored keep the caller's paths.
#[derive(Default)]
pub(super) struct DirectoryRoots {
  paths: RwLock<RootPaths>,
}

impl DirectoryRoots {
  pub(super) fn replace(&self, roots: impl Iterator<Item = InternedPath>) -> SymlinkUpdate {
    let mut resolver = LinkResolver::default();
    let mut paths = RootPaths::default();
    let mut update = SymlinkUpdate::default();
    for root in roots {
      let Some((target, links)) = resolver.directory_root(&root) else {
        continue;
      };
      paths.links.extend(links);
      if target != root {
        paths
          .aliases
          .entry(target.clone())
          .or_default()
          .push(root.clone());
      }
      if cfg!(target_os = "macos")
        && let Some(directory) = target.ancestors().find(|path| path.is_dir())
      {
        update.patterns.push(WatchPattern {
          path: InternedPath::from(directory),
          mode: notify::RecursiveMode::Recursive,
        });
      }
      paths.targets.insert(root, target);
    }
    let mut previous = self.paths.write().expect("directory roots lock poisoned");
    update.retargeted_directories.extend(
      paths
        .targets
        .iter()
        .filter(|(root, target)| {
          previous
            .targets
            .get(*root)
            .is_some_and(|old| old != *target)
        })
        .map(|(root, _)| root.clone()),
    );
    *previous = paths;
    update
  }

  pub(super) fn is_link(&self, path: &InternedPath) -> bool {
    self
      .paths
      .read()
      .expect("directory roots lock poisoned")
      .links
      .contains(path)
  }

  pub(super) fn link_ancestor(&self, path: &InternedPath) -> Option<InternedPath> {
    let paths = self.paths.read().expect("directory roots lock poisoned");
    if paths.links.is_empty() {
      return None;
    }
    path
      .ancestors()
      .skip(1)
      .map(InternedPath::from)
      .find(|path| paths.links.contains(path))
  }

  pub(super) fn aliases(&self, path: &InternedPath) -> InternedPathMap<Vec<InternedPath>> {
    let mut events: InternedPathMap<Vec<InternedPath>> = InternedPathMap::default();
    if cfg!(target_os = "macos") {
      let paths = self.paths.read().expect("directory roots lock poisoned");
      if paths.aliases.is_empty() {
        return events;
      }
      for parent in path.ancestors() {
        if let Some(aliases) = paths.aliases.get(&InternedPath::from(parent)) {
          for root in aliases {
            let relative = path.strip_prefix(parent).expect("path ancestor");
            let alias = if relative.as_os_str().is_empty() {
              root.clone()
            } else {
              InternedPath::from(root.join(relative))
            };
            if alias != *path {
              events.entry(alias).or_default().push(root.clone());
            }
          }
        }
      }
    }
    events
  }
}
