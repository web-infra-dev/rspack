//! Sparse physical-path bookkeeping for `watchOptions.followSymlinks`.
//!
//! Every logical path the caller registers stays exactly as spelled inside
//! [`PathManager`](crate::paths::PathManager); only the OS layer needs physical
//! paths. [`LinkResolver`] expands one logical path component by component, and
//! [`LinkMappings`] records what that expansion costs the physical watch set,
//! so a disk event can be mapped back to the logical subscriptions it serves
//! and released again when the subscription goes.

use std::path::PathBuf;

use rspack_paths::{InternedPath, InternedPathDashSet, InternedPathMap, InternedPathSet};
use rspack_util::fx_hash::FxHashMap;
use smallvec::SmallVec;



/// The physical path a logical path resolves to, the links passed on the way,
/// and the first component that did not exist — so a later `pnpm link` can be
/// noticed without a full rescan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedPath {
  pub target: InternedPath,
  pub links: SmallVec<[InternedPath; 2]>,
  pub first_missing: Option<InternedPath>,
}

impl ResolvedPath {
  pub(crate) fn identity(path: InternedPath) -> Self {
    Self {
      target: path,
      links: SmallVec::new(),
      first_missing: None,
    }
  }

  pub(crate) fn is_identity(&self, path: &InternedPath) -> bool {
    self.links.is_empty() && self.target == *path && self.first_missing.is_none()
  }
}

/// Expands symbolic links component by component, caching every prefix resolved
/// within one watch cycle. Unlike `canonicalize` it stops at the first missing
/// component instead of failing, so a `missing` dependency keeps a stable
/// physical prefix that a newly created link can be resolved against.
#[derive(Default)]
pub(crate) struct LinkResolver {
  cache: InternedPathMap<ResolvedPath>,
  resolving: InternedPathSet,
  follow_symlinks: bool,
}

impl LinkResolver {
  pub(crate) fn new(follow_symlinks: bool) -> Self {
    Self {
      follow_symlinks,
      ..Default::default()
    }
  }

  /// Resolve a logical file or missing dependency. With `followSymlinks: false`
  /// this is the identity, with no system call.
  pub(crate) fn resolve_entry(&mut self, path: &InternedPath) -> ResolvedPath {
    if !self.follow_symlinks {
      return ResolvedPath::identity(path.clone());
    }
    self.resolve(path)
  }

  /// Resolve a logical directory dependency. Without `followSymlinks` this is
  /// the identity everywhere but macOS, where FSEvents reports the physical
  /// spelling of every watched directory.
  pub(crate) fn resolve_directory(&mut self, path: &InternedPath) -> ResolvedPath {
    if self.follow_symlinks {
      return self.resolve(path);
    }
    if !cfg!(target_os = "macos") {
      return ResolvedPath::identity(path.clone());
    }
    match dunce::canonicalize(path) {
      Ok(real) if real.as_path() != path.as_ref() => ResolvedPath {
        target: InternedPath::from(real),
        links: link_prefixes(path),
        first_missing: None,
      },
      _ => ResolvedPath::identity(path.clone()),
    }
  }

  fn resolve(&mut self, path: &InternedPath) -> ResolvedPath {
    if let Some(resolved) = self.cache.get(path) {
      return resolved.clone();
    }
    if !self.resolving.insert(path.clone()) {
      // A link cycle: stop expanding and take the link itself as the end.
      return ResolvedPath::identity(path.clone());
    }
    let resolved = self.resolve_uncached(path);
    self.resolving.remove(path);
    self.cache.insert(path.clone(), resolved.clone());
    resolved
  }

  fn resolve_uncached(&mut self, path: &InternedPath) -> ResolvedPath {
    let Some(parent) = path.parent().map(InternedPath::from) else {
      return ResolvedPath::identity(path.clone());
    };
    if parent == *path {
      return ResolvedPath::identity(path.clone());
    }
    let parent_resolved = self.resolve(&parent);
    let Some(name) = path.file_name() else {
      // POSIX `..` applies to the physical parent, after the preceding links
      // have been expanded — unlike a purely lexical normalization.
      return ResolvedPath {
        target: InternedPath::from(
          parent_resolved
            .target
            .parent()
            .unwrap_or(&parent_resolved.target),
        ),
        links: parent_resolved.links,
        first_missing: parent_resolved.first_missing,
      };
    };
    if let Some(missing) = &parent_resolved.first_missing {
      return ResolvedPath {
        target: InternedPath::from(parent_resolved.target.join(name)),
        links: parent_resolved.links.clone(),
        first_missing: Some(missing.clone()),
      };
    }
    let physical = InternedPath::from(parent_resolved.target.join(name));
    match std::fs::read_link(&physical) {
      Ok(content) => {
        let mut links = parent_resolved.links.clone();
        links.push(physical);
        let next = InternedPath::from(parent_resolved.target.join(dunce::simplified(&content)));
        let mut resolved = self.resolve(&next);
        for link in &resolved.links {
          if !links.contains(link) {
            links.push(link.clone());
          }
        }
        resolved.links = links;
        resolved
      }
      Err(_) => {
        let first_missing = std::fs::symlink_metadata(&physical)
          .is_err()
          .then(|| physical.clone());
        ResolvedPath {
          target: physical,
          links: parent_resolved.links,
          first_missing,
        }
      }
    }
  }
}

/// Every prefix of `path` that is itself a symbolic link, outermost first.
fn link_prefixes(path: &InternedPath) -> SmallVec<[InternedPath; 2]> {
  let mut prefixes = Vec::new();
  let mut current = path.clone();
  while let Some(parent) = current.parent().map(InternedPath::from) {
    if parent == current {
      break;
    }
    prefixes.push(current);
    current = parent;
  }
  prefixes
    .into_iter()
    .rev()
    .filter(|prefix| {
      prefix
        .symlink_metadata()
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    })
    .collect()
}

/// The kind of logical dependency a physical subscription serves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DepKind {
  File = 0,
  Directory = 1,
  Missing = 2,
}


/// Why a physical path is watched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EntrySubscriber {
  /// An explicitly registered file, directory link hop, or missing path.
  Registered {
    kind: DepKind,
    logical_path: InternedPath,
  },
  /// A link found by a context scan pointing at a real file elsewhere.
  ContextFile {
    root_context: InternedPath,
    logical_file: InternedPath,
  },
}

impl EntrySubscriber {
  pub(crate) fn logical_path(&self) -> &InternedPath {
    match self {
      EntrySubscriber::Registered { logical_path, .. } => logical_path,
      EntrySubscriber::ContextFile { logical_file, .. } => logical_file,
    }
  }
}

/// The identity of a watched link: its own mtime and what it points at.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct LinkState {
  mtime: Option<std::time::SystemTime>,
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

/// A physical path set handed to the analyzer, with the same `added` /
/// `removed` diff contract as [`crate::paths::PathTracker`].
#[derive(Default)]
pub(crate) struct PhysicalSet {
  pub all: InternedPathDashSet,
  pub added: InternedPathDashSet,
  pub removed: InternedPathDashSet,
}

impl PhysicalSet {
  /// Replace the whole set, recording what appeared and what went.
  pub(crate) fn replace(&self, next: &InternedPathDashSet) {
    self.added.clear();
    self.removed.clear();
    for path in self.all.iter() {
      if !next.contains(&*path) {
        self.removed.insert(path.clone());
      }
    }
    for path in next.iter() {
      if !self.all.contains(&*path) {
        self.added.insert(path.clone());
      }
    }
    self.all.clear();
    for path in next.iter() {
      self.all.insert(path.clone());
    }
  }

}

/// One logical dependency that must be re-resolved after its link moved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RebindRequest {
  pub kind: DepKind,
  pub logical_path: InternedPath,
}

/// What one [`LinkMappings::rebuild`] changed.
#[derive(Debug, Default)]
pub(crate) struct LinkUpdate {
  /// Logical dependencies whose physical target moved.
  pub retargeted: Vec<RebindRequest>,
  /// Physical directories whose scanned contents belong to an old target.
  pub retargeted_directories: Vec<InternedPath>,
  /// Logical links whose target is ignored: watch the link, not the target.
  pub unfollowed_links: InternedPathSet,
  /// Whether the physical watch set changed at all.
  pub changed: bool,
}

/// The link bookkeeping of one [`crate::paths::PathManager`]. Empty when
/// `followSymlinks` is off and no explicit directory needed canonicalizing.
#[derive(Default)]
pub(crate) struct LinkMappings {
  /// Physical entry path -> the logical subscribers it serves.
  entry_aliases: InternedPathMap<SmallVec<[EntrySubscriber; 1]>>,
  /// Physical directory prefix -> (root context, logical prefix).
  dir_aliases: InternedPathMap<SmallVec<[(InternedPath, InternedPath); 1]>>,
  /// First missing component -> the logical dependencies waiting for it.
  pending_missing_heads: InternedPathMap<SmallVec<[RebindRequest; 1]>>,
  /// (kind, logical path) -> physical paths this registration holds.
  reverse_deps: FxHashMap<(DepKind, InternedPath), SmallVec<[InternedPath; 2]>>,
  /// Root logical context -> physical targets its scan discovered.
  reverse_context_scans: InternedPathMap<SmallVec<[InternedPath; 2]>>,
  /// Physical paths the analyzer watches.
  pub(crate) files: InternedPathDashSet,
  pub(crate) directories: InternedPathDashSet,
  pub(crate) missing: InternedPathDashSet,
  /// Every watched link's last observed state.
  link_states: InternedPathMap<LinkState>,
  /// Logical links whose target is ignored: their target is not watched, and
  /// events reported through them are dropped.
  unfollowed_links: InternedPathSet,
}

impl LinkMappings {
  pub(crate) fn new() -> Self {
    Self::default()
  }


  pub(crate) fn is_empty(&self) -> bool {
    self.entry_aliases.is_empty()
      && self.dir_aliases.is_empty()
      && self.pending_missing_heads.is_empty()
  }

  /// The logical subscribers of a physical entry path.
  pub(crate) fn entry_subscribers(
    &self,
    disk_path: &InternedPath,
  ) -> SmallVec<[EntrySubscriber; 2]> {
    self
      .entry_aliases
      .get(disk_path)
      .map_or_else(SmallVec::new, |subscribers| {
        subscribers.iter().cloned().collect()
      })
  }

  /// The `(root context, logical path, logical link)` triples a physical path
  /// belongs to.
  pub(crate) fn context_subscribers(
    &self,
    disk_path: &InternedPath,
  ) -> SmallVec<[(InternedPath, InternedPath, InternedPath); 2]> {
    let mut contexts = SmallVec::new();
    for ancestor in disk_path.ancestors().skip(1) {
      let ancestor = InternedPath::from(ancestor);
      if let Some(subscribers) = self.dir_aliases.get(&ancestor) {
        let suffix = disk_path
          .strip_prefix(&ancestor)
          .expect("physical ancestor");
        for (root_context, logical_dir) in subscribers {
          let logical = if suffix.as_os_str().is_empty() {
            logical_dir.clone()
          } else {
            InternedPath::from(logical_dir.join(suffix))
          };
          contexts.push((root_context.clone(), logical, logical_dir.clone()));
        }
      }
    }
    contexts
  }

  /// The dependencies waiting for a missing component to appear.
  pub(crate) fn pending_waiters(&self, disk_path: &InternedPath) -> SmallVec<[RebindRequest; 2]> {
    self
      .pending_missing_heads
      .get(disk_path)
      .map_or_else(SmallVec::new, |waiters| {
        waiters.iter().cloned().collect()
      })
  }

  /// The logical dependencies whose physical chain includes this link.
  pub(crate) fn link_waiters(&self, link: &InternedPath) -> SmallVec<[RebindRequest; 2]> {
    let mut waiters = SmallVec::new();
    for ((kind, logical_path), physical) in &self.reverse_deps {
      if physical.iter().any(|path| path == link) {
        waiters.push(RebindRequest {
          kind: *kind,
          logical_path: logical_path.clone(),
        });
      }
    }
    waiters
  }

  /// Whether `path` is a link whose state is being tracked.
  pub(crate) fn tracks_link(&self, path: &InternedPath) -> bool {
    self.link_states.contains_key(path)
  }


  /// Whether `path` is a link whose target is ignored, so nothing behind it
  /// is reported.
  pub(crate) fn is_unfollowed(&self, path: &InternedPath) -> bool {
    self.unfollowed_links.contains(path)
  }

  /// Cheap guard before walking a path's ancestors.
  pub(crate) fn unfollowed_is_empty(&self) -> bool {
    self.unfollowed_links.is_empty()
  }

  /// Compare the link's current state with the recorded one, advancing the
  /// record. Only a tracked link ever reports a change here.
  /// The logical path a physical context entry maps back to, if any.
  /// Drop every mapping. Recorded link states stay, so a retarget can still be
  /// told from an attribute-only event.
  fn clear_mappings(&mut self) {
    self.entry_aliases.clear();
    self.dir_aliases.clear();
    self.pending_missing_heads.clear();
    self.reverse_deps.clear();
  }

  #[allow(clippy::too_many_arguments)]
  fn bind(
    &mut self,
    kind: DepKind,
    logical_path: &InternedPath,
    resolved: &ResolvedPath,
    target_ignored: bool,
    files: &mut InternedPathDashSet,
    directories: &mut InternedPathDashSet,
    missing: &mut InternedPathDashSet,
  ) {
    let directory = kind == DepKind::Directory;
    let mut bound: SmallVec<[InternedPath; 2]> = SmallVec::new();
    for link in &resolved.links {
      self
        .entry_aliases
        .entry(link.clone())
        .or_default()
        .push(EntrySubscriber::Registered {
          kind,
          logical_path: logical_path.clone(),
        });
      files.insert(link.clone());
      bound.push(link.clone());
      self
        .link_states
        .entry(link.clone())
        .or_insert_with(|| LinkState::read(link));
    }
    if resolved.target != *logical_path && !target_ignored {
      if directory {
        self
          .dir_aliases
          .entry(resolved.target.clone())
          .or_default()
          .push((logical_path.clone(), logical_path.clone()));
        directories.insert(resolved.target.clone());
      } else {
        self
          .entry_aliases
          .entry(resolved.target.clone())
          .or_default()
          .push(EntrySubscriber::Registered {
            kind,
            logical_path: logical_path.clone(),
          });
        files.insert(resolved.target.clone());
      }
      bound.push(resolved.target.clone());
    } else if resolved.target == *logical_path && directory {
      directories.insert(logical_path.clone());
    } else if resolved.target == *logical_path && kind == DepKind::Missing {
      missing.insert(logical_path.clone());
    } else if resolved.target == *logical_path {
      files.insert(logical_path.clone());
    }
    if let Some(head) = &resolved.first_missing {
      self
        .pending_missing_heads
        .entry(head.clone())
        .or_default()
        .push(RebindRequest {
          kind,
          logical_path: logical_path.clone(),
        });
      bound.push(head.clone());
      if directory {
        directories.insert(head.clone());
      } else if kind == DepKind::Missing {
        missing.insert(head.clone());
      } else {
        files.insert(head.clone());
      }
    }
    if !bound.is_empty() {
      self
        .reverse_deps
        .insert((kind, logical_path.clone()), bound);
    }
  }

  /// Rebuild every physical subscription from the current logical sets. Runs
  /// once per watch cycle and again after a link retarget.
  pub(crate) fn rebuild(
    &mut self,
    resolver: &mut LinkResolver,
    files: &InternedPathSet,
    directories: &InternedPathSet,
    missing: &InternedPathSet,
    ignored_targets: &InternedPathSet,
    unfollowed_links: &InternedPathSet,
  ) -> LinkUpdate {
    let mut update = LinkUpdate::default();
    let previous_reverse = self.reverse_deps.clone();
    // Context scan results survive a rebuild: a later `watch()` cycle must not
    // forget the physical targets a context link points at.
    let context_targets: Vec<(InternedPath, InternedPath, InternedPath, bool)> = self
      .reverse_context_scans
      .iter()
      .flat_map(|(root, targets)| {
        targets.iter().map(|target| {
          (
            root.clone(),
            target.clone(),
            target.clone(),
            target.is_dir(),
          )
        })
      })
      .collect();
    let entry_context: Vec<(InternedPath, InternedPath, InternedPath)> = self
      .entry_aliases
      .iter()
      .flat_map(|(physical, subscribers)| {
        subscribers
          .iter()
          .filter_map(move |subscriber| match subscriber {
            EntrySubscriber::ContextFile {
              root_context,
              logical_file,
            } => Some((root_context.clone(), logical_file.clone(), physical.clone())),
            EntrySubscriber::Registered { .. } => None,
          })
      })
      .collect();
    let dir_context: Vec<(InternedPath, InternedPath, InternedPath)> = self
      .dir_aliases
      .iter()
      .flat_map(|(physical, subscribers)| {
        subscribers.iter().map(move |(root_context, logical_dir)| {
          (root_context.clone(), logical_dir.clone(), physical.clone())
        })
      })
      .collect();
    self.clear_mappings();

    let mut next_files = InternedPathDashSet::default();
    let mut next_directories = InternedPathDashSet::default();
    let mut next_missing = InternedPathDashSet::default();
    update.unfollowed_links = unfollowed_links.clone();
    self.unfollowed_links = unfollowed_links.clone();

    for logical in files {
      let resolved = resolver.resolve_entry(logical);
      self.bind(
        DepKind::File,
        logical,
        &resolved,
        ignored_targets.contains(&resolved.target),
        &mut next_files,
        &mut next_directories,
        &mut next_missing,
      );
    }
    for logical in directories {
      let resolved = resolver.resolve_directory(logical);
      self.bind(
        DepKind::Directory,
        logical,
        &resolved,
        ignored_targets.contains(&resolved.target),
        &mut next_files,
        &mut next_directories,
        &mut next_missing,
      );
    }
    for logical in missing {
      let resolved = resolver.resolve_entry(logical);
      self.bind(
        DepKind::Missing,
        logical,
        &resolved,
        ignored_targets.contains(&resolved.target),
        &mut next_files,
        &mut next_directories,
        &mut next_missing,
      );
    }

    // Context scan results survive a rebuild: a later `watch()` cycle must not
    // forget the physical targets a context link points at.
    for (root_context, logical_file, physical) in entry_context {
      self
        .entry_aliases
        .entry(physical.clone())
        .or_default()
        .push(EntrySubscriber::ContextFile {
          root_context: root_context.clone(),
          logical_file,
        });
      next_files.insert(physical.clone());
      self
        .reverse_context_scans
        .entry(root_context)
        .or_default()
        .push(physical);
    }
    for (root_context, logical_dir, physical) in dir_context {
      self
        .dir_aliases
        .entry(physical.clone())
        .or_default()
        .push((root_context.clone(), logical_dir));
      next_directories.insert(physical.clone());
      self
        .reverse_context_scans
        .entry(root_context)
        .or_default()
        .push(physical);
    }
    let _ = context_targets;

    for ((kind, logical_path), previous) in &previous_reverse {
      let current = self.reverse_deps.get(&(*kind, logical_path.clone()));
      if current != Some(previous) {
        update.retargeted.push(RebindRequest {
          kind: *kind,
          logical_path: logical_path.clone(),
        });
        if *kind == DepKind::Directory {
          for target in previous {
            if target.is_dir() {
              update.retargeted_directories.push(target.clone());
            }
          }
        }
      }
    }

    self.files = next_files;
    self.directories = next_directories;
    self.missing = next_missing;
    update.changed =
      !self.files.is_empty() || !self.directories.is_empty() || !self.missing.is_empty();
    update
  }
  /// Record the physical target of a link discovered by a context scan.
  pub(crate) fn bind_context_target(
    &mut self,
    root_context: &InternedPath,
    logical_path: &InternedPath,
    resolved: &ResolvedPath,
    directory: bool,
    files: &mut InternedPathDashSet,
    directories: &mut InternedPathDashSet,
  ) {
    if directory {
      self
        .dir_aliases
        .entry(resolved.target.clone())
        .or_default()
        .push((root_context.clone(), logical_path.clone()));
      directories.insert(resolved.target.clone());
    } else {
      self
        .entry_aliases
        .entry(resolved.target.clone())
        .or_default()
        .push(EntrySubscriber::ContextFile {
          root_context: root_context.clone(),
          logical_file: logical_path.clone(),
        });
      files.insert(resolved.target.clone());
    }
    self
      .reverse_context_scans
      .entry(root_context.clone())
      .or_default()
      .push(resolved.target.clone());
  }

}
