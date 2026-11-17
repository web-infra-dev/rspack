use std::{
  path::{Component, PathBuf},
  sync::{Arc, RwLock},
  time::SystemTime,
};

use rspack_paths::{InternedPath, InternedPathMap, InternedPathSet};
use rspack_util::time::current_time;

use crate::{
  WatchPattern,
  paths::PathManager,
  time::{mtime_accuracy, mtime_safe_time},
};

struct Resolution {
  target: InternedPath,
  links: InternedPathSet,
  is_link: bool,
}

/// A watch cycle shares parent resolutions, including parents of missing files.
/// Unlike canonicalize, read_link does not require the final target to exist.
#[derive(Default)]
pub(crate) struct LinkResolver {
  cache: InternedPathMap<Arc<Resolution>>,
  resolving: InternedPathSet,
}

impl LinkResolver {
  pub(crate) fn directory_root(
    &mut self,
    path: &InternedPath,
  ) -> Option<(InternedPath, Vec<InternedPath>)> {
    let target = self.resolve(path)?.target.clone();
    let links = path
      .ancestors()
      .map(InternedPath::from)
      .filter(|path| self.cache.get(path).is_some_and(|entry| entry.is_link))
      .collect();
    Some((target, links))
  }

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
        is_link: false,
      }));
    };
    let parent = self.resolve(&InternedPath::from(parent))?;
    let real = match path.components().next_back()? {
      Component::ParentDir => InternedPath::from(parent.target.parent().unwrap_or(&parent.target)),
      _ => InternedPath::from(parent.target.join(path.file_name()?)),
    };
    let mut links = parent.links.clone();
    let content = std::fs::read_link(&real);
    let is_link = content.is_ok();
    let target = if let Ok(content) = content {
      links.insert(real);
      let target = self.resolve(&InternedPath::from(
        parent.target.join(dunce::simplified(&content)),
      ))?;
      links.extend(target.links.iter().cloned());
      target.target.clone()
    } else {
      real
    };
    Some(Arc::new(Resolution {
      target,
      links,
      is_link,
    }))
  }
}

/// Cloned so startup scans can run after releasing the subscription lock.
#[derive(Clone)]
pub(crate) struct LinkScan {
  pub links: Vec<InternedPath>,
  pub target: Option<InternedPath>,
}

#[derive(Default)]
struct WatchedLink {
  aliases: InternedPathSet,
  state: LinkState,
}

#[derive(Default, PartialEq)]
struct LinkState {
  mtime: Option<SystemTime>,
  target: Option<PathBuf>,
}

impl LinkState {
  fn read(path: &InternedPath) -> Self {
    Self {
      mtime: path
        .symlink_metadata()
        .and_then(|metadata| metadata.modified())
        .ok(),
      target: std::fs::read_link(path).ok(),
    }
  }
}

impl WatchedLink {
  fn is_unchanged(&mut self, path: &InternedPath, ignore_when_equal: bool) -> bool {
    let previous = std::mem::replace(&mut self.state, LinkState::read(path));
    let unchanged = previous == self.state;
    match self.state.mtime {
      Some(mtime) => {
        mtime_accuracy(mtime);
        unchanged && (ignore_when_equal || mtime_safe_time(mtime) < current_time())
      }
      None => unchanged,
    }
  }
}

pub(crate) enum LinkEvent {
  Disk,
  Injected,
  Recheck,
}

pub(crate) enum LinkSubscriptionKind {
  // File and missing dependencies can also refer to directory links.
  ExplicitEntry,
  ExplicitDirectory,
  ContextFile,
  ContextDirectory,
}

#[derive(Default)]
pub(crate) struct LinkSubscriptions {
  targets: InternedPathMap<InternedPathSet>,
  watched_children: InternedPathMap<InternedPathSet>,
  directories: InternedPathMap<InternedPathSet>,
  resolved_directories: InternedPathMap<InternedPath>,
  directory_targets: InternedPathMap<InternedPath>,
  context_files: InternedPathSet,
  link_scans: InternedPathMap<LinkScan>,
  links: InternedPathMap<WatchedLink>,
  patterns: Vec<WatchPattern>,
}

impl LinkSubscriptions {
  fn resolved_directory_entry(&self, path: &InternedPath) -> Option<InternedPath> {
    path.ancestors().skip(1).find_map(|parent| {
      self
        .resolved_directories
        .get(&InternedPath::from(parent))
        .map(|target| {
          InternedPath::from(target.join(path.strip_prefix(parent).expect("path ancestor")))
        })
    })
  }

  pub(crate) async fn add(
    &mut self,
    resolver: &mut LinkResolver,
    path: InternedPath,
    kind: LinkSubscriptionKind,
    manager: &PathManager,
  ) {
    let directory = matches!(
      kind,
      LinkSubscriptionKind::ExplicitDirectory | LinkSubscriptionKind::ContextDirectory
    );
    let nested = matches!(
      kind,
      LinkSubscriptionKind::ContextFile | LinkSubscriptionKind::ContextDirectory
    );
    let Some(resolution) = resolver.resolve(&path) else {
      return;
    };
    if resolution.links.is_empty() && resolution.target == path {
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
    }
    for link in &resolution.links {
      if !nested && manager.is_ignored_path(link).await {
        continue;
      }
      self
        .links
        .entry(link.clone())
        .or_default()
        .aliases
        .insert(path.clone());
      self.index_ancestors(link);
      self.watch(link, false);
      // ReadDirectoryChangesW does not report moves of its watched directory.
      #[cfg(target_os = "windows")]
      if let Some(parent) = link.parent() {
        self.watch(&InternedPath::from(parent), false);
      }
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
    // Watchpack watches nested directories by alias, but file targets by real path.
    let target_ignored =
      (!nested || !directory) && manager.is_ignored_path(&resolution.target).await;
    if !nested {
      let links = resolution
        .links
        .iter()
        .filter(|link| {
          self
            .links
            .get(*link)
            .is_some_and(|link| link.aliases.contains(&path))
        })
        .cloned()
        .collect();
      self.link_scans.insert(
        path.clone(),
        LinkScan {
          links,
          target: (!target_ignored).then(|| resolution.target.clone()),
        },
      );
    }
    if target_ignored {
      return;
    }
    if nested && !directory {
      self.context_files.insert(path.clone());
    }
    self
      .targets
      .entry(resolution.target.clone())
      .or_default()
      .insert(path.clone());
    self.index_ancestors(&resolution.target);
    self.watch(&resolution.target, directory);
    if directory {
      // FSEvents can report only the parent when a target tree is removed.
      self.watch(&resolution.target, false);
    }
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

  fn index_ancestors(&mut self, path: &InternedPath) {
    let mut child = path.clone();
    for ancestor in path.ancestors().skip(1) {
      let parent = InternedPath::from(ancestor);
      if !self
        .watched_children
        .entry(parent.clone())
        .or_default()
        .insert(child)
      {
        break;
      }
      child = parent;
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

#[derive(Default)]
pub(crate) struct Symlinks {
  // Published as a whole so event processing never observes a half-built index.
  subscriptions: RwLock<LinkSubscriptions>,
}

pub(crate) struct SymlinkAlias {
  pub path: InternedPath,
  pub changed_link: Option<InternedPath>,
  pub resolved_path: Option<InternedPath>,
}

#[derive(Default)]
pub(crate) struct SymlinkUpdate {
  pub patterns: Vec<WatchPattern>,
  pub newly_subscribed: Vec<InternedPath>,
  pub new_context_files: Vec<InternedPath>,
  /// Only retargeting discards old entries; new subscriptions need them to detect deletions.
  pub retargeted_directories: Vec<InternedPath>,
  pub needs_context_scan: bool,
}

impl Symlinks {
  pub(crate) fn is_unchanged_link_event(&self, path: &InternedPath) -> bool {
    let mut subscriptions = self
      .subscriptions
      .write()
      .expect("symlink subscriptions lock poisoned");
    let Some(link) = subscriptions.links.get_mut(path) else {
      return false;
    };
    link.is_unchanged(path, false)
  }

  pub(crate) fn has_target_subscription(&self, path: &InternedPath) -> bool {
    self
      .subscriptions
      .read()
      .expect("symlink subscriptions lock poisoned")
      .link_scans
      .get(path)
      .is_some_and(|scan| scan.target.is_some())
  }

  pub(crate) fn link_scan(&self, path: &InternedPath) -> Option<LinkScan> {
    self
      .subscriptions
      .read()
      .expect("symlink subscriptions lock poisoned")
      .link_scans
      .get(path)
      .cloned()
  }

  pub(crate) fn subscription_path(
    &self,
    path: &InternedPath,
    changed_link: Option<&InternedPath>,
  ) -> Option<InternedPath> {
    let subscriptions = self
      .subscriptions
      .read()
      .expect("symlink subscriptions lock poisoned");
    let Some(scan) = subscriptions.link_scans.get(path) else {
      return Some(path.clone());
    };
    match changed_link {
      Some(link) => scan.links.contains(link).then(|| link.clone()),
      None => scan.target.clone(),
    }
  }

  pub(crate) fn resolved_directory(&self, path: &InternedPath) -> Option<InternedPath> {
    self
      .subscriptions
      .read()
      .expect("symlink subscriptions lock poisoned")
      .resolved_directories
      .get(path)
      .cloned()
  }

  pub(crate) fn context_directories(&self) -> InternedPathSet {
    let subscriptions = self
      .subscriptions
      .read()
      .expect("symlink subscriptions lock poisoned");
    subscriptions
      .directory_targets
      .keys()
      .filter(|path| !subscriptions.resolved_directories.contains_key(*path))
      .cloned()
      .collect()
  }

  pub(crate) fn resolved_directory_entry(&self, path: &InternedPath) -> Option<InternedPath> {
    self
      .subscriptions
      .read()
      .expect("symlink subscriptions lock poisoned")
      .resolved_directory_entry(path)
  }

  pub(crate) fn context_files(&self) -> InternedPathSet {
    self
      .subscriptions
      .read()
      .expect("symlink subscriptions lock poisoned")
      .context_files
      .clone()
  }

  pub(crate) fn is_context_file(&self, path: &InternedPath) -> bool {
    self
      .subscriptions
      .read()
      .expect("symlink subscriptions lock poisoned")
      .context_files
      .contains(path)
  }

  pub(crate) fn replace(&self, mut subscriptions: LinkSubscriptions) -> SymlinkUpdate {
    let patterns = std::mem::take(&mut subscriptions.patterns);
    let mut previous = self
      .subscriptions
      .write()
      .expect("symlink subscriptions lock poisoned");
    for (path, link) in &mut subscriptions.links {
      link.state = match previous.links.get_mut(path) {
        Some(old) => std::mem::take(&mut old.state),
        None => LinkState::read(path),
      };
    }
    let newly_subscribed = subscriptions
      .link_scans
      .iter()
      .filter(|(path, scan)| {
        previous.link_scans.get(*path).is_none_or(|old| {
          scan.target != old.target || scan.links.iter().any(|link| !old.links.contains(link))
        })
      })
      .map(|(path, _)| path.clone())
      .collect();
    let mut new_context_files = Vec::new();
    for (target, aliases) in &subscriptions.targets {
      for alias in aliases {
        if subscriptions.context_files.contains(alias)
          && (!previous.context_files.contains(alias)
            || !previous
              .targets
              .get(target)
              .is_some_and(|old| old.contains(alias)))
        {
          new_context_files.push(alias.clone());
        }
      }
    }
    let mut retargeted_directories = Vec::new();
    let mut has_new_directories = false;
    for (path, target) in &subscriptions.directory_targets {
      match previous.directory_targets.get(path) {
        Some(old) if old != target => retargeted_directories.push(path.clone()),
        None => has_new_directories = true,
        _ => {}
      }
    }
    let needs_context_scan =
      has_new_directories || !retargeted_directories.is_empty() || !new_context_files.is_empty();
    *previous = subscriptions;
    SymlinkUpdate {
      patterns,
      newly_subscribed,
      new_context_files,
      retargeted_directories,
      needs_context_scan,
    }
  }

  /// Changing a link invalidates its dependencies even when the replacement
  /// target has the same modification time.
  pub(crate) fn aliases(
    &self,
    path: &InternedPath,
    include_descendants: bool,
    event: LinkEvent,
  ) -> Vec<SymlinkAlias> {
    let mut subscriptions = self
      .subscriptions
      .write()
      .expect("symlink subscriptions lock poisoned");
    let include_descendants =
      include_descendants || (subscriptions.watched_children.contains_key(path) && !path.exists());
    let mut aliases = InternedPathMap::default();
    let mut pending = vec![path.clone()];
    while let Some(watched) = pending.pop() {
      if let Some(targets) = subscriptions.targets.get(&watched) {
        for target in targets {
          aliases.entry(target.clone()).or_insert(None);
        }
      }
      if let Some(link) = subscriptions.links.get_mut(&watched)
        && match event {
          LinkEvent::Injected => true,
          // Direct disk events already passed the link-state check at ingress.
          LinkEvent::Disk => watched == *path || !link.is_unchanged(&watched, false),
          LinkEvent::Recheck => !link.is_unchanged(&watched, true),
        }
      {
        for target in &link.aliases {
          aliases.insert(target.clone(), Some(watched.clone()));
        }
      }
      if include_descendants && let Some(children) = subscriptions.watched_children.get(&watched) {
        pending.extend(children.iter().cloned());
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
          aliases.entry(alias).or_insert(None);
        }
      }
    }
    aliases
      .into_iter()
      .map(|(alias, changed_link)| {
        let resolved_path = if changed_link.is_some() {
          None
        } else {
          subscriptions.resolved_directory_entry(&alias)
        };
        SymlinkAlias {
          path: alias,
          changed_link,
          resolved_path,
        }
      })
      .collect()
  }
}
