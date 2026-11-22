use std::{
  path::{Component, Path, PathBuf},
  time::SystemTime,
};

use rspack_paths::{InternedPath, InternedPathMap, InternedPathSet};

use super::PathTracker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DepKind {
  File = 0,
  Directory = 1,
  Missing = 2,
}

impl DepKind {
  pub const ALL: [Self; 3] = [Self::File, Self::Directory, Self::Missing];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LinkState {
  pub mtime: Option<SystemTime>,
  pub target: Option<PathBuf>,
}

impl LinkState {
  pub fn read(path: &Path) -> Self {
    Self {
      mtime: std::fs::symlink_metadata(path)
        .and_then(|m| m.modified().or_else(|_| m.created()))
        .ok(),
      target: std::fs::read_link(path).ok(),
    }
  }
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedPath {
  pub target: InternedPath,
  pub links: Vec<InternedPath>,
  pub first_missing: Option<InternedPath>,
}

pub(crate) struct LinkResolver {
  follow_symlinks: bool,
  cache: InternedPathMap<ResolvedPath>,
  resolving: InternedPathSet,
}

impl LinkResolver {
  pub fn new(follow_symlinks: bool) -> Self {
    Self {
      follow_symlinks,
      cache: InternedPathMap::default(),
      resolving: InternedPathSet::default(),
    }
  }

  pub fn resolve(&mut self, path: &InternedPath) -> ResolvedPath {
    if !self.follow_symlinks {
      let normalized = if has_dot_or_parent(path.as_path()) {
        normalize_lexical(path.as_path())
      } else {
        path.as_path().to_path_buf()
      };
      return ResolvedPath {
        target: canonicalize_ancestor_prefix(&normalized, 1),
        links: Vec::new(),
        first_missing: None,
      };
    }

    if let Some(cached) = self.cache.get(path) {
      return cached.clone();
    }
    let resolved = self.resolve_uncached(path);
    if self.resolving.is_empty() {
      self.cache.insert(path.clone(), resolved.clone());
    }
    resolved
  }

  pub fn resolve_directory_root(&mut self, dir: &InternedPath) -> ResolvedPath {
    if self.follow_symlinks {
      return self.resolve(dir);
    }
    let normalized = if has_dot_or_parent(dir.as_path()) {
      normalize_lexical(dir.as_path())
    } else {
      dir.as_path().to_path_buf()
    };
    ResolvedPath {
      target: canonicalize_ancestor_prefix(&normalized, 0),
      links: Vec::new(),
      first_missing: None,
    }
  }

  fn resolve_uncached(&mut self, path: &InternedPath) -> ResolvedPath {
    let Some(parent) = path.parent().filter(|parent| *parent != path.as_path()) else {
      return ResolvedPath {
        target: path.clone(),
        links: Vec::new(),
        first_missing: None,
      };
    };
    let Some(last_component) = path.components().next_back() else {
      return ResolvedPath {
        target: path.clone(),
        links: Vec::new(),
        first_missing: None,
      };
    };

    match last_component {
      Component::CurDir => self.resolve(&InternedPath::from(parent)),
      Component::ParentDir => {
        let mut parent_res = self.resolve(&InternedPath::from(parent));
        parent_res.target = InternedPath::from(
          parent_res
            .target
            .parent()
            .unwrap_or(parent_res.target.as_path()),
        );
        parent_res
      }
      Component::Normal(name) => {
        let parent_res = self.resolve(&InternedPath::from(parent));
        let real = InternedPath::from(parent_res.target.join(name));
        match std::fs::read_link(real.as_path()) {
          Ok(link_content) => {
            let mut links = parent_res.links.clone();
            if !is_immutable_system_link(real.as_path()) {
              push_unique(&mut links, real.clone());
            }
            if !self.resolving.insert(real.clone()) {
              return ResolvedPath {
                target: real,
                links,
                first_missing: parent_res.first_missing,
              };
            }
            let next = InternedPath::from(parent_res.target.join(dunce::simplified(&link_content)));
            let target_res = self.resolve(&next);
            self.resolving.remove(&real);
            for link in target_res.links {
              push_unique(&mut links, link);
            }
            ResolvedPath {
              target: target_res.target,
              links,
              first_missing: parent_res.first_missing.or(target_res.first_missing),
            }
          }
          Err(err) => {
            let first_missing = parent_res.first_missing.or_else(|| {
              (err.kind() != std::io::ErrorKind::InvalidInput
                && std::fs::symlink_metadata(real.as_path()).is_err())
              .then(|| real.clone())
            });
            ResolvedPath {
              target: real,
              links: parent_res.links,
              first_missing,
            }
          }
        }
      }
      _ => ResolvedPath {
        target: path.clone(),
        links: Vec::new(),
        first_missing: None,
      },
    }
  }
}

fn is_immutable_system_link(path: &Path) -> bool {
  #[cfg(target_os = "macos")]
  {
    matches!(path.to_str(), Some("/var" | "/tmp" | "/etc"))
  }
  #[cfg(not(target_os = "macos"))]
  {
    let _ = path;
    false
  }
}

fn canonicalize_ancestor_prefix(path: &Path, skip_self: usize) -> InternedPath {
  #[cfg(windows)]
  {
    for ancestor in path.ancestors().skip(skip_self) {
      if ancestor.as_os_str().is_empty() {
        break;
      }
      if !ancestor.exists() {
        continue;
      }
      if !ancestor
        .ancestors()
        .any(|item| !item.as_os_str().is_empty() && item.is_symlink())
      {
        return InternedPath::from(path);
      }
      let resolved = LinkResolver::new(true)
        .resolve(&InternedPath::from(ancestor))
        .target;
      if !resolved.exists() {
        return InternedPath::from(path);
      }
      if let Ok(rel) = path.strip_prefix(ancestor) {
        return if rel.as_os_str().is_empty() {
          resolved
        } else {
          InternedPath::from(resolved.join(rel))
        };
      }
      break;
    }
    InternedPath::from(path)
  }
  #[cfg(not(windows))]
  {
    for ancestor in path.ancestors().skip(skip_self) {
      if ancestor.as_os_str().is_empty() {
        break;
      }
      if let Ok(canonical) = dunce::canonicalize(ancestor) {
        if canonical == ancestor {
          return InternedPath::from(path);
        }
        if let Ok(rel) = path.strip_prefix(ancestor) {
          return if rel.as_os_str().is_empty() {
            InternedPath::from(canonical)
          } else {
            InternedPath::from(canonical.join(rel))
          };
        }
        break;
      }
    }
    InternedPath::from(path)
  }
}

pub(crate) fn has_dot_or_parent(path: &Path) -> bool {
  path
    .components()
    .any(|c| matches!(c, Component::CurDir | Component::ParentDir))
}

pub(crate) fn normalize_lexical(path: &Path) -> PathBuf {
  let mut out = PathBuf::new();
  for component in path.components() {
    match component {
      Component::CurDir => {}
      Component::ParentDir => {
        let _ = out.pop();
      }
      other => out.push(other.as_os_str()),
    }
  }
  out
}

pub(crate) fn push_unique<T: PartialEq>(vec: &mut Vec<T>, item: T) {
  if !vec.contains(&item) {
    vec.push(item);
  }
}

fn remove_map_item<T: PartialEq>(
  map: &mut InternedPathMap<Vec<T>>,
  key: &InternedPath,
  item: &T,
) -> bool {
  let emptied = if let Some(list) = map.get_mut(key) {
    if let Some(pos) = list.iter().position(|cur| cur == item) {
      list.remove(pos);
    }
    list.is_empty()
  } else {
    false
  };
  if emptied {
    map.remove(key);
  }
  emptied
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EntrySubscriber {
  Registered {
    kind: DepKind,
    logical_path: InternedPath,
    is_link_hop: bool,
  },
  ContextFile {
    root_context: InternedPath,
    logical_file: InternedPath,
  },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ContextMapping {
  pub root_context: InternedPath,
  pub record_prefix: InternedPath,
  pub filter_prefix: InternedPath,
}

#[derive(Debug, Clone)]
pub(crate) struct BoundDep {
  pub target: InternedPath,
  pub target_active: bool,
  pub links: Vec<InternedPath>,
  pub first_missing: Option<InternedPath>,
}

impl BoundDep {
  pub fn is_identity(&self, logical_path: &InternedPath) -> bool {
    self.target == *logical_path
      && self.target_active
      && self.links.is_empty()
      && self.first_missing.is_none()
  }

  pub fn retained_items(&self) -> Vec<InternedPath> {
    let mut items = Vec::with_capacity(usize::from(self.target_active) + self.links.len());
    if self.target_active {
      items.push(self.target.clone());
    }
    for link in &self.links {
      push_unique(&mut items, link.clone());
    }
    items
  }
}

#[derive(Debug, Default)]
pub(crate) struct ContextScanBindings {
  pub watched_dirs: InternedPathSet,
  pub aliased_dirs: InternedPathMap<Vec<ContextMapping>>,
  pub watched_files: InternedPathMap<InternedPathSet>,
  pub scanned_links: InternedPathMap<LinkState>,
}

#[derive(Default)]
pub(crate) struct LinkMappings {
  pub entry_aliases: InternedPathMap<Vec<EntrySubscriber>>,
  pub dir_aliases: InternedPathMap<Vec<ContextMapping>>,
  pub pending_missing_heads: InternedPathMap<Vec<(DepKind, InternedPath)>>,
  pub bound_deps: [InternedPathMap<BoundDep>; 3],
  pub link_states: InternedPathMap<LinkState>,
  pub link_ref_counts: InternedPathMap<u32>,
  pub children_by_dir: InternedPathMap<InternedPathSet>,
  pub reverse_context_scans: InternedPathMap<ContextScanBindings>,
  pub dirty_deps: [InternedPathSet; 3],
  pub watch_refs: [InternedPathMap<u32>; 3],
  pub root_dir_refs: InternedPathMap<u32>,
}

impl LinkMappings {
  pub fn get_bound(&self, kind: DepKind, logical_path: &InternedPath) -> Option<&BoundDep> {
    self.bound_deps[kind as usize].get(logical_path)
  }

  pub fn recorded_size(&self) -> usize {
    self.entry_aliases.len()
      + self.dir_aliases.len()
      + self.link_states.len()
      + self.reverse_context_scans.len()
  }

  pub fn registered_size(&self) -> usize {
    self.bound_deps.iter().map(|m| m.len()).sum::<usize>()
      + self.pending_missing_heads.len()
      + self.children_by_dir.len()
      + self.watch_refs.iter().map(|m| m.len()).sum::<usize>()
  }

  pub fn is_physically_watched_file_or_missing(&self, path: &InternedPath) -> bool {
    self.watch_refs[DepKind::File as usize].contains_key(path)
      || self.watch_refs[DepKind::Missing as usize].contains_key(path)
      || self.link_ref_counts.contains_key(path)
  }

  pub fn is_physically_watched_dir(&self, path: &InternedPath) -> bool {
    self.watch_refs[DepKind::Directory as usize].contains_key(path)
      || self.root_dir_refs.contains_key(path)
  }

  pub fn add_watch_ref(
    &mut self,
    kind: DepKind,
    path: &InternedPath,
    trackers: [&PathTracker; 3],
  ) -> bool {
    let idx = kind as usize;
    let count = self.watch_refs[idx].entry(path.clone()).or_default();
    *count += 1;
    if *count == 1 {
      trackers[idx].add(path.clone());
      let root_dir = if kind == DepKind::Directory {
        Some(path.clone())
      } else {
        path.parent().map(InternedPath::from)
      };
      if let Some(dir) = root_dir {
        *self.root_dir_refs.entry(dir).or_default() += 1;
      }
      return true;
    }
    false
  }

  pub fn remove_watch_ref(
    &mut self,
    kind: DepKind,
    path: &InternedPath,
    trackers: [&PathTracker; 3],
    evicted: &mut Vec<InternedPath>,
  ) {
    let idx = kind as usize;
    let removed = self.watch_refs[idx].get_mut(path).is_some_and(|count| {
      *count -= 1;
      *count == 0
    });
    if removed {
      self.watch_refs[idx].remove(path);
      trackers[idx].remove(path.clone());
      evicted.push(path.clone());
      let root_candidate = if kind == DepKind::Directory {
        Some(path.clone())
      } else {
        path.parent().map(InternedPath::from)
      };
      if let Some(dir) = root_candidate
        && let Some(rc) = self.root_dir_refs.get_mut(&dir)
      {
        *rc -= 1;
        if *rc == 0 {
          self.root_dir_refs.remove(&dir);
          evicted.push(dir);
        }
      }
    }
  }

  fn retain_link_state(&mut self, link: &InternedPath) {
    let count = self.link_ref_counts.entry(link.clone()).or_default();
    *count += 1;
    if *count == 1 {
      self
        .link_states
        .insert(link.clone(), LinkState::read(link.as_path()));
    }
  }

  fn release_link_state(&mut self, link: &InternedPath) {
    if self.link_ref_counts.get_mut(link).is_some_and(|count| {
      *count -= 1;
      *count == 0
    }) {
      self.link_ref_counts.remove(link);
      self.link_states.remove(link);
    }
  }

  fn index_physical_path(&mut self, path: &InternedPath) {
    let mut child = path.clone();
    while let Some(parent) = child.parent().map(InternedPath::from) {
      if !self
        .children_by_dir
        .entry(parent.clone())
        .or_default()
        .insert(child)
      {
        break;
      }
      child = parent;
    }
  }

  fn unindex_physical_path(&mut self, path: &InternedPath) {
    let mut child = path.clone();
    while !self.entry_aliases.contains_key(&child)
      && !self.pending_missing_heads.contains_key(&child)
      && !self.children_by_dir.contains_key(&child)
    {
      let Some(parent) = child.parent().map(InternedPath::from) else {
        return;
      };
      let emptied = self
        .children_by_dir
        .get_mut(&parent)
        .is_some_and(|siblings| siblings.remove(&child) && siblings.is_empty());
      if !emptied {
        return;
      }
      self.children_by_dir.remove(&parent);
      child = parent;
    }
  }

  fn insert_entry_subscriber(&mut self, physical: &InternedPath, sub: EntrySubscriber) {
    push_unique(self.entry_aliases.entry(physical.clone()).or_default(), sub);
    self.index_physical_path(physical);
  }

  fn remove_entry_subscriber(&mut self, physical: &InternedPath, sub: &EntrySubscriber) {
    if remove_map_item(&mut self.entry_aliases, physical, sub) {
      self.unindex_physical_path(physical);
    }
  }

  fn insert_dir_alias(&mut self, physical_dir: &InternedPath, mapping: ContextMapping) {
    push_unique(
      self.dir_aliases.entry(physical_dir.clone()).or_default(),
      mapping,
    );
  }

  fn remove_dir_alias(&mut self, physical_dir: &InternedPath, mapping: &ContextMapping) {
    remove_map_item(&mut self.dir_aliases, physical_dir, mapping);
  }

  pub fn bind_dep(
    &mut self,
    kind: DepKind,
    logical_path: InternedPath,
    bound: BoundDep,
    follow_symlinks: bool,
    trackers: [&PathTracker; 3],
    evicted: &mut Vec<InternedPath>,
  ) {
    self.unbind_dep(kind, &logical_path, trackers, evicted);

    if bound.is_identity(&logical_path) {
      self.add_watch_ref(kind, &logical_path, trackers);
      return;
    }

    for link in &bound.links {
      self.add_watch_ref(DepKind::File, link, trackers);
      self.retain_link_state(link);
      self.insert_entry_subscriber(
        link,
        EntrySubscriber::Registered {
          kind,
          logical_path: logical_path.clone(),
          is_link_hop: true,
        },
      );
    }

    if bound.target_active {
      self.add_watch_ref(kind, &bound.target, trackers);
      match kind {
        DepKind::File | DepKind::Missing => {
          self.insert_entry_subscriber(
            &bound.target,
            EntrySubscriber::Registered {
              kind,
              logical_path: logical_path.clone(),
              is_link_hop: false,
            },
          );
        }
        DepKind::Directory => {
          let filter_prefix = if follow_symlinks {
            bound.target.clone()
          } else {
            logical_path.clone()
          };
          if bound.target != logical_path || filter_prefix != logical_path {
            self.insert_dir_alias(
              &bound.target,
              ContextMapping {
                root_context: logical_path.clone(),
                record_prefix: logical_path.clone(),
                filter_prefix,
              },
            );
          }
        }
      }
    }

    if let Some(head) = &bound.first_missing {
      self.add_watch_ref(DepKind::Missing, head, trackers);
      push_unique(
        self.pending_missing_heads.entry(head.clone()).or_default(),
        (kind, logical_path.clone()),
      );
      self.index_physical_path(head);
    }

    self.bound_deps[kind as usize].insert(logical_path, bound);
  }

  pub fn unbind_dep(
    &mut self,
    kind: DepKind,
    logical_path: &InternedPath,
    trackers: [&PathTracker; 3],
    evicted: &mut Vec<InternedPath>,
  ) -> bool {
    let idx = kind as usize;
    self.dirty_deps[idx].remove(logical_path);
    let Some(bound) = self.bound_deps[idx].remove(logical_path) else {
      if self.watch_refs[idx].contains_key(logical_path) {
        self.remove_watch_ref(kind, logical_path, trackers, evicted);
        return true;
      }
      return false;
    };

    for link in &bound.links {
      self.remove_watch_ref(DepKind::File, link, trackers, evicted);
      self.release_link_state(link);
      self.remove_entry_subscriber(
        link,
        &EntrySubscriber::Registered {
          kind,
          logical_path: logical_path.clone(),
          is_link_hop: true,
        },
      );
    }

    if bound.target_active {
      self.remove_watch_ref(kind, &bound.target, trackers, evicted);
      match kind {
        DepKind::File | DepKind::Missing => {
          self.remove_entry_subscriber(
            &bound.target,
            &EntrySubscriber::Registered {
              kind,
              logical_path: logical_path.clone(),
              is_link_hop: false,
            },
          );
        }
        DepKind::Directory => {
          for filter_prefix in [bound.target.clone(), logical_path.clone()] {
            self.remove_dir_alias(
              &bound.target,
              &ContextMapping {
                root_context: logical_path.clone(),
                record_prefix: logical_path.clone(),
                filter_prefix,
              },
            );
          }
        }
      }
    }

    if let Some(head) = &bound.first_missing {
      self.remove_watch_ref(DepKind::Missing, head, trackers, evicted);
      if remove_map_item(
        &mut self.pending_missing_heads,
        head,
        &(kind, logical_path.clone()),
      ) {
        self.unindex_physical_path(head);
      }
    }
    true
  }

  pub fn add_context_dir_watch(
    &mut self,
    root_context: &InternedPath,
    physical_dir: InternedPath,
    alias: Option<ContextMapping>,
    trackers: [&PathTracker; 3],
  ) -> bool {
    let (add_dir, add_alias) = {
      let bindings = self
        .reverse_context_scans
        .entry(root_context.clone())
        .or_default();
      let add_dir = bindings.watched_dirs.insert(physical_dir.clone());
      let add_alias = alias.and_then(|mapping| {
        let list = bindings
          .aliased_dirs
          .entry(physical_dir.clone())
          .or_default();
        if list.contains(&mapping) {
          None
        } else {
          list.push(mapping.clone());
          Some(mapping)
        }
      });
      (add_dir, add_alias)
    };
    let newly_watched = if add_dir {
      self.add_watch_ref(DepKind::Directory, &physical_dir, trackers)
    } else {
      false
    };
    if let Some(mapping) = add_alias {
      self.insert_dir_alias(&physical_dir, mapping);
    }
    newly_watched
  }

  pub fn add_context_file_watch(
    &mut self,
    root_context: &InternedPath,
    physical_file: InternedPath,
    logical_file: InternedPath,
    trackers: [&PathTracker; 3],
  ) -> bool {
    let inserted = self
      .reverse_context_scans
      .entry(root_context.clone())
      .or_default()
      .watched_files
      .entry(physical_file.clone())
      .or_default()
      .insert(logical_file.clone());
    if !inserted {
      return false;
    }
    let newly_watched = self.add_watch_ref(DepKind::File, &physical_file, trackers);
    self.insert_entry_subscriber(
      &physical_file,
      EntrySubscriber::ContextFile {
        root_context: root_context.clone(),
        logical_file,
      },
    );
    newly_watched
  }

  pub fn record_context_scanned_link(&mut self, root_context: &InternedPath, link: InternedPath) {
    let state = LinkState::read(link.as_path());
    self
      .reverse_context_scans
      .entry(root_context.clone())
      .or_default()
      .scanned_links
      .insert(link, state);
  }

  pub fn clear_context_scan(
    &mut self,
    root_context: &InternedPath,
    trackers: [&PathTracker; 3],
    evicted: &mut Vec<InternedPath>,
  ) {
    let Some(bindings) = self.reverse_context_scans.remove(root_context) else {
      return;
    };
    for dir in bindings.watched_dirs {
      self.remove_watch_ref(DepKind::Directory, &dir, trackers, evicted);
    }
    for (dir, mappings) in bindings.aliased_dirs {
      for mapping in mappings {
        self.remove_dir_alias(&dir, &mapping);
      }
    }
    for (physical_file, logical_files) in bindings.watched_files {
      for logical_file in logical_files {
        self.remove_watch_ref(DepKind::File, &physical_file, trackers, evicted);
        self.remove_entry_subscriber(
          &physical_file,
          &EntrySubscriber::ContextFile {
            root_context: root_context.clone(),
            logical_file,
          },
        );
      }
    }
  }

  pub fn clear_context_entries_under_prefix(
    &mut self,
    logical_prefix: &Path,
    trackers: [&PathTracker; 3],
    evicted: &mut Vec<InternedPath>,
  ) {
    let mut empty_roots = Vec::new();
    let mut dirs_to_unwatch = Vec::new();
    let mut aliases_to_remove = Vec::new();
    let mut files_to_remove = Vec::new();

    for (root_ctx, bindings) in &mut self.reverse_context_scans {
      let mut removed_aliased_physicals = InternedPathSet::default();
      bindings.aliased_dirs.retain(|phys, mappings| {
        mappings.retain(|mapping| {
          if mapping.record_prefix.starts_with(logical_prefix) {
            aliases_to_remove.push((phys.clone(), mapping.clone()));
            removed_aliased_physicals.insert(phys.clone());
            false
          } else {
            true
          }
        });
        !mappings.is_empty()
      });
      bindings.watched_dirs.retain(|phys| {
        let still_aliased = bindings.aliased_dirs.contains_key(phys);
        let remove = (removed_aliased_physicals.contains(phys) && !still_aliased)
          || (!still_aliased && phys.starts_with(logical_prefix));
        if remove {
          dirs_to_unwatch.push(phys.clone());
        }
        !remove
      });
      bindings.watched_files.retain(|phys, logical_files| {
        logical_files.retain(|logical_file| {
          let remove = logical_file.starts_with(logical_prefix);
          if remove {
            files_to_remove.push((root_ctx.clone(), phys.clone(), logical_file.clone()));
          }
          !remove
        });
        !logical_files.is_empty()
      });
      bindings
        .scanned_links
        .retain(|link, _| !link.starts_with(logical_prefix));
      if bindings.watched_dirs.is_empty()
        && bindings.aliased_dirs.is_empty()
        && bindings.watched_files.is_empty()
        && bindings.scanned_links.is_empty()
      {
        empty_roots.push(root_ctx.clone());
      }
    }

    for root in empty_roots {
      self.reverse_context_scans.remove(&root);
    }
    for dir in dirs_to_unwatch {
      self.remove_watch_ref(DepKind::Directory, &dir, trackers, evicted);
    }
    for (dir, mapping) in aliases_to_remove {
      self.remove_dir_alias(&dir, &mapping);
    }
    for (root_context, phys, logical_file) in files_to_remove {
      self.remove_watch_ref(DepKind::File, &phys, trackers, evicted);
      self.remove_entry_subscriber(
        &phys,
        &EntrySubscriber::ContextFile {
          root_context,
          logical_file,
        },
      );
    }
  }

  pub fn collect_physical_descendants(&self, dir: &InternedPath) -> Vec<InternedPath> {
    let mut out = Vec::new();
    if !self.children_by_dir.contains_key(dir) {
      return out;
    }
    let mut stack = vec![dir.clone()];
    while let Some(curr) = stack.pop() {
      if let Some(children) = self.children_by_dir.get(&curr) {
        for child in children {
          if self.entry_aliases.contains_key(child)
            || self.pending_missing_heads.contains_key(child)
          {
            out.push(child.clone());
          }
          if self.children_by_dir.contains_key(child) {
            stack.push(child.clone());
          }
        }
      }
    }
    out
  }
}
