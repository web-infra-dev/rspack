use std::{collections::hash_map::Entry, hash::BuildHasherDefault};

use rspack_collections::IdentifierSet;
use rspack_paths::{InternedPath, InternedPathList, InternedPathMap};
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;
use ustr::IdentityHasher;

use crate::{DependencyId, ModuleIdentifier, utils::incremental_info::IncrementalInfo};

/// Tracks the modules and the dependency lists that currently reference a path.
#[derive(Debug, Default)]
struct PathResourceIds {
  modules: IdentifierSet,
  groups: GroupRefs,
}

/// The dependency lists that mention one path.
#[derive(Debug)]
enum GroupRefs {
  Small(SmallVec<[InternedPathList; 1]>),
  /// Paths that belong to very many distinct lists - resolver candidates are
  /// probed from many different factorize results - switch to a set: removing
  /// a reference from a dense vector scans every entry, so dissolving one of
  /// those lists would cost one full scan per path it mentions and degrade
  /// into quadratic work, while a set removes in constant time.
  Hot(Box<FxHashSet<InternedPathList>>),
}

impl Default for GroupRefs {
  fn default() -> Self {
    Self::Small(SmallVec::new())
  }
}

impl GroupRefs {
  /// Above this many lists sharing a path, removing must not scan.
  const HOT_THRESHOLD: usize = 16;

  fn push(&mut self, list: InternedPathList) {
    match self {
      GroupRefs::Small(lists) => {
        lists.push(list);
        if lists.len() > Self::HOT_THRESHOLD {
          let lists = std::mem::take(lists);
          *self = GroupRefs::Hot(Box::new(lists.into_iter().collect()));
        }
      }
      GroupRefs::Hot(lists) => {
        lists.insert(list);
      }
    }
  }

  fn remove(&mut self, list: &InternedPathList) {
    match self {
      GroupRefs::Small(lists) => lists.retain(|item| item != list),
      GroupRefs::Hot(lists) => {
        lists.remove(list);
      }
    }
  }

  fn is_empty(&self) -> bool {
    match self {
      GroupRefs::Small(lists) => lists.is_empty(),
      GroupRefs::Hot(lists) => lists.is_empty(),
    }
  }

  fn iter(&self) -> GroupRefsIter<'_> {
    match self {
      GroupRefs::Small(lists) => GroupRefsIter::Small(lists.iter()),
      GroupRefs::Hot(lists) => GroupRefsIter::Hot(lists.iter()),
    }
  }
}

/// Borrowed iteration over a path's group references.
#[derive(Debug)]
enum GroupRefsIter<'a> {
  Small(std::slice::Iter<'a, InternedPathList>),
  Hot(std::collections::hash_set::Iter<'a, InternedPathList>),
}

impl<'a> Iterator for GroupRefsIter<'a> {
  type Item = &'a InternedPathList;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    match self {
      GroupRefsIter::Small(lists) => lists.next(),
      GroupRefsIter::Hot(lists) => lists.next(),
    }
  }
}

impl PathResourceIds {
  fn is_empty(&self) -> bool {
    self.modules.is_empty() && self.groups.is_empty()
  }
}

/// The dependencies recorded with one exact interned path list.
type DependencyGroups = FxHashMap<InternedPathList, FxHashSet<DependencyId>>;

/// The resources that reference one path.
pub struct RelatedResourceIds<'a> {
  entry: &'a PathResourceIds,
  groups: &'a DependencyGroups,
}

impl<'a> RelatedResourceIds<'a> {
  /// The modules that built with this path.
  pub fn modules(&self) -> &IdentifierSet {
    &self.entry.modules
  }

  /// Iterates the dependency ids whose factorize result lists this path.
  ///
  /// A dependency id is yielded once per list it was recorded with. Callers
  /// keep adding and removing a dependency balanced, so normally every id
  /// appears exactly once.
  pub fn dependencies(&self) -> DependencyIdsIter<'a> {
    DependencyIdsIter {
      refs: self.entry.groups.iter(),
      groups: self.groups,
      current: None,
    }
  }
}

/// Iterator over the dependency ids recorded for one path, across the lists
/// that mention it.
#[derive(Debug)]
pub struct DependencyIdsIter<'a> {
  refs: GroupRefsIter<'a>,
  groups: &'a DependencyGroups,
  current: Option<std::collections::hash_set::Iter<'a, DependencyId>>,
}

impl Iterator for DependencyIdsIter<'_> {
  type Item = DependencyId;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if let Some(ids) = self.current.as_mut() {
        if let Some(id) = ids.next() {
          return Some(*id);
        }
        self.current = None;
      }
      let list = self.refs.next()?;
      self.current = self.groups.get(list).map(|ids| ids.iter());
    }
  }
}

/// Used to count file usage and track which modules/dependencies use each file
#[derive(Debug, Default)]
pub struct FileCounter {
  inner: InternedPathMap<PathResourceIds>,
  /// The dependency ids grouped by the interned list that recorded them.
  ///
  /// A dependency registers its file list at every path it mentions, but the
  /// ids themselves only belong to the list. Keying them by the interned list
  /// (equal lists share one handle) stores equal content once instead of once
  /// per referenced path, and adding or removing a dependency touches a single
  /// set no matter how many paths its list mentions.
  groups: DependencyGroups,
  incremental_info: IncrementalInfo<InternedPath, BuildHasherDefault<IdentityHasher>>,
}

impl FileCounter {
  /// Adds the files a module built with.
  pub fn add_module_files<'a>(
    &mut self,
    module: ModuleIdentifier,
    paths: impl IntoIterator<Item = &'a InternedPath>,
  ) {
    for path in paths {
      let entry = self.inner.entry(path.clone()).or_default();
      if entry.is_empty() {
        self.incremental_info.mark_as_add(path);
      }
      entry.modules.insert(module);
    }
  }

  /// Removes the files a module built with.
  pub fn remove_module_files<'a>(
    &mut self,
    module: ModuleIdentifier,
    paths: impl IntoIterator<Item = &'a InternedPath>,
  ) {
    for path in paths {
      let Some(entry) = self.inner.get_mut(path) else {
        panic!("unable to remove untracked file {}", path.to_string_lossy());
      };
      if !entry.modules.remove(&module) {
        panic!(
          "unable to remove path '{}' with module '{}', it has not been added.",
          path.to_string_lossy(),
          module,
        )
      }
      if entry.is_empty() {
        self.incremental_info.mark_as_remove(path);
        self.inner.remove(path);
      }
    }
  }

  /// Adds the interned file list that a dependency factorized with.
  pub fn add_dependency_files(&mut self, dependency: DependencyId, list: &InternedPathList) {
    if list.is_empty() {
      return;
    }
    match self.groups.entry(list.clone()) {
      Entry::Occupied(mut group) => {
        group.get_mut().insert(dependency);
      }
      Entry::Vacant(group) => {
        for path in list.as_slice() {
          let entry = self.inner.entry(path.clone()).or_default();
          if entry.is_empty() {
            self.incremental_info.mark_as_add(path);
          }
          entry.groups.push(list.clone());
        }
        group.insert(std::iter::once(dependency).collect());
      }
    }
  }

  /// Removes the interned file list that a dependency factorized with.
  pub fn remove_dependency_files(&mut self, dependency: DependencyId, list: &InternedPathList) {
    if list.is_empty() {
      return;
    }
    let Some(group) = self.groups.get_mut(list) else {
      panic!("unable to remove dependency {dependency:?}, its list is not tracked.");
    };
    if !group.remove(&dependency) {
      panic!("unable to remove dependency {dependency:?}, it has not been added.");
    }
    if group.is_empty() {
      self.groups.remove(list);
      for path in list.as_slice() {
        let entry = self
          .inner
          .get_mut(path)
          .expect("a path of a tracked list should be tracked");
        entry.groups.remove(list);
        if entry.is_empty() {
          self.incremental_info.mark_as_remove(path);
          self.inner.remove(path);
        }
      }
    }
  }

  /// Get the file that has been used
  pub fn files(&self) -> impl Iterator<Item = &InternedPath> {
    self.inner.keys()
  }

  /// Get the resource ids (modules/dependencies) that use a specific file
  pub fn related_resource_ids(&self, path: &InternedPath) -> Option<RelatedResourceIds<'_>> {
    self.inner.get(path).map(|entry| RelatedResourceIds {
      entry,
      groups: &self.groups,
    })
  }

  /// reset incremental info
  pub fn reset_incremental_info(&mut self) {
    self.incremental_info.reset();
  }

  /// Added files compared to the `files()` when call reset_incremental_info
  pub fn added_files(&self) -> impl Iterator<Item = &InternedPath> {
    self.incremental_info.added().iter()
  }

  /// Updated files compared to the `files()` when call reset_incremental_info
  pub fn updated_files(&self) -> impl Iterator<Item = &InternedPath> {
    self.incremental_info.updated().iter()
  }

  /// Removed files compared to the `files()` when call reset_incremental_info
  pub fn removed_files(&self) -> impl Iterator<Item = &InternedPath> {
    self.incremental_info.removed().iter()
  }
}

#[cfg(test)]
mod test {
  use rspack_paths::{InternedPath, InternedPathList, InternedPathSet};

  use super::FileCounter;
  use crate::{DependencyId, ModuleIdentifier};

  #[test]
  fn file_counter_is_available() {
    let mut counter = FileCounter::default();
    let file_a = InternedPath::from(std::path::PathBuf::from("/a"));
    let file_b = InternedPath::from(std::path::PathBuf::from("/b"));
    let file_list_a = {
      let mut list = InternedPathSet::default();
      list.insert(file_a.clone());
      list
    };
    let file_list_b = {
      let mut list = InternedPathSet::default();
      list.insert(file_b.clone());
      list
    };
    let file_list_all = {
      let mut list = InternedPathSet::default();
      list.insert(file_a);
      list.insert(file_b);
      list
    };

    let module_1: ModuleIdentifier = "A".into();
    let module_2: ModuleIdentifier = "B".into();

    counter.add_module_files(module_1, &file_list_all);
    counter.add_module_files(module_2, &file_list_a);
    assert_eq!(counter.files().count(), 2);
    assert_eq!(counter.added_files().count(), 2);
    assert_eq!(counter.removed_files().count(), 0);

    // test repeated additions
    counter.add_module_files(module_1, &file_list_all);
    assert_eq!(counter.files().count(), 2);
    assert_eq!(counter.added_files().count(), 2);
    assert_eq!(counter.removed_files().count(), 0);

    counter.remove_module_files(module_1, &file_list_a);
    assert_eq!(counter.files().count(), 2);
    assert_eq!(counter.added_files().count(), 2);
    assert_eq!(counter.removed_files().count(), 0);

    counter.remove_module_files(module_1, &file_list_b);
    assert_eq!(counter.files().count(), 1);
    assert_eq!(counter.added_files().count(), 1);
    assert_eq!(counter.removed_files().count(), 0);

    counter.remove_module_files(module_2, &file_list_a);
    assert_eq!(counter.files().count(), 0);
    assert_eq!(counter.added_files().count(), 0);
    assert_eq!(counter.removed_files().count(), 0);
  }

  #[test]
  #[should_panic]
  fn file_counter_remove_file_with_panic() {
    let mut counter = FileCounter::default();
    let file_a = InternedPath::from(std::path::PathBuf::from("/a"));
    let file_list_a = {
      let mut list = InternedPathSet::default();
      list.insert(file_a);
      list
    };
    let module: ModuleIdentifier = "A".into();
    counter.remove_module_files(module, &file_list_a);
  }

  #[test]
  fn file_counter_reset_incremental_info() {
    let mut counter = FileCounter::default();
    let file_a = InternedPath::from(std::path::PathBuf::from("/a"));
    let file_list_a = {
      let mut list = InternedPathSet::default();
      list.insert(file_a);
      list
    };
    let module_1: ModuleIdentifier = "A".into();
    let module_2: ModuleIdentifier = "B".into();

    counter.add_module_files(module_1, &file_list_a);
    assert_eq!(counter.added_files().count(), 1);
    assert_eq!(counter.removed_files().count(), 0);

    counter.reset_incremental_info();
    assert_eq!(counter.added_files().count(), 0);
    assert_eq!(counter.removed_files().count(), 0);

    counter.remove_module_files(module_1, &file_list_a);
    assert_eq!(counter.added_files().count(), 0);
    assert_eq!(counter.removed_files().count(), 1);

    counter.add_module_files(module_1, &file_list_a);
    assert_eq!(counter.added_files().count(), 0);
    assert_eq!(counter.removed_files().count(), 0);

    counter.reset_incremental_info();
    assert_eq!(counter.added_files().count(), 0);
    assert_eq!(counter.removed_files().count(), 0);

    counter.add_module_files(module_2, &file_list_a);
    assert_eq!(counter.added_files().count(), 0);
    assert_eq!(counter.removed_files().count(), 0);

    counter.remove_module_files(module_1, &file_list_a);
    assert_eq!(counter.added_files().count(), 0);
    assert_eq!(counter.removed_files().count(), 0);

    counter.remove_module_files(module_2, &file_list_a);
    assert_eq!(counter.added_files().count(), 0);
    assert_eq!(counter.removed_files().count(), 1);
  }

  #[test]
  fn file_counter_tracks_modules_and_dependencies_separately() {
    let mut counter = FileCounter::default();
    let file = InternedPath::from(std::path::PathBuf::from("/a"));
    let dependency_list = InternedPathList::from_vec(vec![file.clone()]);
    let module: ModuleIdentifier = "A".into();
    let dependency = DependencyId::from(1);

    counter.add_module_files(module, [&file]);
    counter.add_dependency_files(dependency, &dependency_list);

    let ids = counter
      .related_resource_ids(&file)
      .expect("should track path resource ids");
    assert_eq!(ids.modules().len(), 1);
    assert_eq!(ids.dependencies().count(), 1);

    counter.remove_module_files(module, [&file]);
    let ids = counter
      .related_resource_ids(&file)
      .expect("should keep dependency after removing module");
    assert!(ids.modules().is_empty());
    assert_eq!(ids.dependencies().count(), 1);
  }
}
