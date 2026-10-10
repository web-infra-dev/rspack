mod resource_id;

use std::hash::BuildHasherDefault;

use rspack_collections::IdentifierSet;
use rspack_paths::{InternedPath, InternedPathMap};
use rustc_hash::FxHashSet;
use ustr::IdentityHasher;

pub use self::resource_id::ResourceId;
use crate::{DependencyId, utils::incremental_info::IncrementalInfo};

/// The dependency ids that reference one path.
///
/// While make adds files the ids live in a hash set for fast lookups. Once make
/// stopped, [`FileCounter::freeze`] collapses the set into an unsorted, exactly
/// sized vector: a hash table keeps a control byte per bucket plus up to twice
/// the live element count in growth slack, which dominates the reverse index of
/// a large production build. Frozen lists only support iteration; a later make
/// pass must thaw them back into hash sets before modifying them.
#[derive(Debug, Default)]
enum DependencyIds {
  #[default]
  Empty,
  Set(FxHashSet<DependencyId>),
  /// Unsorted, deduplicated, and sized close to `len`. Read-only until thawed.
  Frozen(Vec<DependencyId>),
}

impl DependencyIds {
  #[inline]
  fn len(&self) -> usize {
    match self {
      DependencyIds::Empty => 0,
      DependencyIds::Set(set) => set.len(),
      DependencyIds::Frozen(ids) => ids.len(),
    }
  }

  #[inline]
  fn is_empty(&self) -> bool {
    self.len() == 0
  }

  fn insert(&mut self, id: DependencyId) -> bool {
    match self {
      DependencyIds::Empty => {
        *self = DependencyIds::Set(std::iter::once(id).collect());
        true
      }
      DependencyIds::Set(set) => set.insert(id),
      DependencyIds::Frozen(_) => unreachable!("dependency ids must be thawed before insertion"),
    }
  }

  fn remove(&mut self, id: &DependencyId) -> bool {
    match self {
      DependencyIds::Empty => false,
      DependencyIds::Set(set) => set.remove(id),
      DependencyIds::Frozen(_) => unreachable!("dependency ids must be thawed before removal"),
    }
  }

  fn thaw(&mut self) {
    if let DependencyIds::Frozen(ids) = self {
      *self = DependencyIds::Set(std::mem::take(ids).into_iter().collect());
    }
  }

  fn iter(&self) -> DependencyIdsIter<'_> {
    match self {
      DependencyIds::Empty => DependencyIdsIter::Empty,
      DependencyIds::Set(set) => DependencyIdsIter::Set(set.iter()),
      DependencyIds::Frozen(ids) => DependencyIdsIter::Frozen(ids.iter()),
    }
  }

  /// Whether the ids are stored as a hash set, i.e. still need collapsing.
  fn is_hash_form(&self) -> bool {
    matches!(self, DependencyIds::Set(_))
  }

  /// Collapses a working hash set into the dense iteration-only form. Idempotent.
  fn freeze(&mut self) {
    if let DependencyIds::Set(set) = self {
      *self = DependencyIds::Frozen(set.drain().collect());
    }
  }
}

/// Iterator over the dependency ids of one path. Yields owned ids because the
/// storage may switch between the hash and the dense representation.
#[derive(Debug)]
pub enum DependencyIdsIter<'a> {
  Empty,
  Set(std::collections::hash_set::Iter<'a, DependencyId>),
  Frozen(std::slice::Iter<'a, DependencyId>),
}

impl Iterator for DependencyIdsIter<'_> {
  type Item = DependencyId;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    match self {
      DependencyIdsIter::Empty => None,
      DependencyIdsIter::Set(iter) => iter.next().copied(),
      DependencyIdsIter::Frozen(iter) => iter.next().copied(),
    }
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    match self {
      DependencyIdsIter::Empty => (0, Some(0)),
      DependencyIdsIter::Set(iter) => iter.size_hint(),
      DependencyIdsIter::Frozen(iter) => iter.size_hint(),
    }
  }
}

impl ExactSizeIterator for DependencyIdsIter<'_> {}

/// Tracks the modules and dependencies that currently reference a path.
#[derive(Debug, Default)]
pub struct PathResourceIds {
  modules: IdentifierSet,
  dependencies: DependencyIds,
}

impl PathResourceIds {
  fn insert(&mut self, resource_id: &ResourceId) -> bool {
    match resource_id {
      ResourceId::Module(module_id) => self.modules.insert(*module_id),
      ResourceId::Dependency(dependency_id) => {
        self.dependencies.thaw();
        self.dependencies.insert(*dependency_id)
      }
    }
  }

  fn remove(&mut self, resource_id: &ResourceId) -> bool {
    match resource_id {
      ResourceId::Module(module_id) => self.modules.remove(module_id),
      ResourceId::Dependency(dependency_id) => {
        self.dependencies.thaw();
        self.dependencies.remove(dependency_id)
      }
    }
  }

  pub fn is_empty(&self) -> bool {
    self.modules.is_empty() && self.dependencies.is_empty()
  }

  pub fn modules(&self) -> &IdentifierSet {
    &self.modules
  }

  /// Iterates the dependency ids. The storage is a hash set while make builds
  /// the index and an unsorted slice once the counter has been frozen.
  pub fn dependencies(&self) -> DependencyIdsIter<'_> {
    self.dependencies.iter()
  }

  /// Whether the dependency ids are stored as a hash set.
  fn dependencies_in_hash_form(&self) -> bool {
    self.dependencies.is_hash_form()
  }

  /// Collapses the working hash set into the dense iteration-only form.
  fn freeze(&mut self) {
    self.dependencies.freeze();
  }
}

/// Used to count file usage and track which modules/dependencies use each file
#[derive(Debug, Default)]
pub struct FileCounter {
  inner: InternedPathMap<PathResourceIds>,
  incremental_info: IncrementalInfo<InternedPath, BuildHasherDefault<IdentityHasher>>,
  /// A frozen counter rejects mutations until make or an explicit module
  /// rebuild thaws it.
  frozen: bool,
  /// Paths whose dependency ids were put into the hash form during this make
  /// and therefore still need to be collapsed by [`FileCounter::freeze`]. The
  /// list is filled where a mutation creates or thaws a hash set, so freezing
  /// never has to rescan a map whose entries are almost all already dense.
  hash_form_paths: Vec<InternedPath>,
}

impl FileCounter {
  /// Collapse the reverse index into its dense post-make form. While make
  /// builds the index it needs hash lookups; the retained hash tables then
  /// carry one control byte per bucket plus up to twice the live element count
  /// in growth slack. Freezing at the end of make stores the ids densely in
  /// arbitrary order. Only the paths a mutation put into the hash form
  /// are visited, so a rebuild collapses just what it touched and later
  /// rebuilds thaw only the sets they mutate. Adding or removing files is
  /// rejected until [`FileCounter::thaw`] allows another graph update.
  pub fn freeze(&mut self) {
    for path in std::mem::take(&mut self.hash_form_paths) {
      if let Some(ids) = self.inner.get_mut(&path) {
        ids.freeze();
      }
    }
    self.frozen = true;
  }

  /// Allows graph updates to modify the counter and returns whether it was
  /// frozen. Dependency lists stay dense until a mutation needs to turn them
  /// back into hash sets.
  pub fn thaw(&mut self) -> bool {
    std::mem::take(&mut self.frozen)
  }

  /// Add batch [`PathBuf`] to counter
  ///
  /// It will add resource_id at the PathBuf in inner hashmap
  pub fn add_files<'a>(
    &mut self,
    resource_id: &ResourceId,
    paths: impl IntoIterator<Item = &'a InternedPath>,
  ) {
    assert!(!self.frozen, "cannot add files to a frozen file counter");
    for path in paths {
      let list = self.inner.entry(path.clone()).or_default();
      if list.is_empty() {
        self.incremental_info.mark_as_add(path);
      }
      let in_hash_form = list.dependencies_in_hash_form();
      // multiple additions are allowed without additional checks to see if the addition was successful
      list.insert(resource_id);
      if !in_hash_form && list.dependencies_in_hash_form() {
        self.hash_form_paths.push(path.clone());
      }
    }
  }

  /// Remove batch [`PathBuf`] from counter
  ///
  /// It will remove resource_id at the PathBuf in inner hashmap
  ///
  /// If the PathBuf resource_id is empty after reduction, the record will be deleted
  /// If PathBuf does not exist, panic will occur.
  pub fn remove_files<'a>(
    &mut self,
    resource_id: &ResourceId,
    paths: impl IntoIterator<Item = &'a InternedPath>,
  ) {
    assert!(
      !self.frozen,
      "cannot remove files from a frozen file counter"
    );
    for path in paths {
      let Some(list) = self.inner.get_mut(path) else {
        panic!("unable to remove untracked file {}", path.to_string_lossy());
      };
      let in_hash_form = list.dependencies_in_hash_form();
      if !list.remove(resource_id) {
        panic!(
          "unable to remove path '{}' with resource_id '{:?}', it has not been added.",
          path.to_string_lossy(),
          resource_id,
        )
      }
      if !in_hash_form && list.dependencies_in_hash_form() {
        self.hash_form_paths.push(path.clone());
      }
      if list.is_empty() {
        self.incremental_info.mark_as_remove(path);
        self.inner.remove(path);
      }
    }
  }

  /// Get the file that has been used
  pub fn files(&self) -> impl Iterator<Item = &InternedPath> {
    self.inner.keys()
  }

  /// Get the resource ids (modules/dependencies) that use a specific file
  pub fn related_resource_ids(&self, path: &InternedPath) -> Option<&PathResourceIds> {
    self.inner.get(path)
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
  use rspack_paths::{InternedPath, InternedPathSet};

  use super::{FileCounter, ResourceId};
  use crate::DependencyId;
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

    let resource_1 = ResourceId::Module("A".into());
    let resource_2 = ResourceId::Module("B".into());

    counter.add_files(&resource_1, &file_list_all);
    counter.add_files(&resource_2, &file_list_a);
    assert_eq!(counter.files().count(), 2);
    assert_eq!(counter.added_files().count(), 2);
    assert_eq!(counter.removed_files().count(), 0);

    // test repeated additions
    counter.add_files(&resource_1, &file_list_all);
    assert_eq!(counter.files().count(), 2);
    assert_eq!(counter.added_files().count(), 2);
    assert_eq!(counter.removed_files().count(), 0);

    counter.remove_files(&resource_1, &file_list_a);
    assert_eq!(counter.files().count(), 2);
    assert_eq!(counter.added_files().count(), 2);
    assert_eq!(counter.removed_files().count(), 0);

    counter.remove_files(&resource_1, &file_list_b);
    assert_eq!(counter.files().count(), 1);
    assert_eq!(counter.added_files().count(), 1);
    assert_eq!(counter.removed_files().count(), 0);

    counter.remove_files(&resource_2, &file_list_a);
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
    let resource = ResourceId::Module("A".into());
    counter.remove_files(&resource, &file_list_a);
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
    let resource_1 = ResourceId::Module("A".into());
    let resource_2 = ResourceId::Module("B".into());

    counter.add_files(&resource_1, &file_list_a);
    assert_eq!(counter.added_files().count(), 1);
    assert_eq!(counter.removed_files().count(), 0);

    counter.reset_incremental_info();
    assert_eq!(counter.added_files().count(), 0);
    assert_eq!(counter.removed_files().count(), 0);

    counter.remove_files(&resource_1, &file_list_a);
    assert_eq!(counter.added_files().count(), 0);
    assert_eq!(counter.removed_files().count(), 1);

    counter.add_files(&resource_1, &file_list_a);
    assert_eq!(counter.added_files().count(), 0);
    assert_eq!(counter.removed_files().count(), 0);

    counter.reset_incremental_info();
    assert_eq!(counter.added_files().count(), 0);
    assert_eq!(counter.removed_files().count(), 0);

    counter.add_files(&resource_2, &file_list_a);
    assert_eq!(counter.added_files().count(), 0);
    assert_eq!(counter.removed_files().count(), 0);

    counter.remove_files(&resource_1, &file_list_a);
    assert_eq!(counter.added_files().count(), 0);
    assert_eq!(counter.removed_files().count(), 0);

    counter.remove_files(&resource_2, &file_list_a);
    assert_eq!(counter.added_files().count(), 0);
    assert_eq!(counter.removed_files().count(), 1);
  }

  #[test]
  fn file_counter_tracks_modules_and_dependencies_separately() {
    let mut counter = FileCounter::default();
    let file = InternedPath::from(std::path::PathBuf::from("/a"));
    let file_list = {
      let mut list = InternedPathSet::default();
      list.insert(file.clone());
      list
    };
    let module = ResourceId::Module("A".into());
    let dependency = ResourceId::Dependency(DependencyId::from(1));

    counter.add_files(&module, &file_list);
    counter.add_files(&dependency, &file_list);

    let ids = counter
      .related_resource_ids(&file)
      .expect("should track path resource ids");
    assert_eq!(ids.modules().len(), 1);
    assert_eq!(ids.dependencies().len(), 1);

    counter.remove_files(&module, &file_list);
    let ids = counter
      .related_resource_ids(&file)
      .expect("should keep dependency after removing module");
    assert!(ids.modules().is_empty());
    assert_eq!(ids.dependencies().len(), 1);
  }
}
