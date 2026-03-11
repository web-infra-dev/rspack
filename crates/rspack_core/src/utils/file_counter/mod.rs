mod resource_id;

use std::hash::BuildHasherDefault;

use rspack_collections::{IdentifierSet, UkeySet};
use rspack_paths::{ArcPath, ArcPathMap, ArcPathSet};
use ustr::IdentityHasher;

pub use self::resource_id::ResourceId;
use crate::{DependencyId, utils::incremental_info::IncrementalInfo};

/// Tracks the modules and dependencies that currently reference a path.
#[derive(Debug, Default)]
pub struct PathResourceIds {
  modules: IdentifierSet,
  dependencies: UkeySet<DependencyId>,
}

impl PathResourceIds {
  fn insert(&mut self, resource_id: &ResourceId) -> bool {
    match resource_id {
      ResourceId::Module(module_id) => self.modules.insert(*module_id),
      ResourceId::Dependency(dependency_id) => self.dependencies.insert(*dependency_id),
    }
  }

  fn remove(&mut self, resource_id: &ResourceId) -> bool {
    match resource_id {
      ResourceId::Module(module_id) => self.modules.remove(module_id),
      ResourceId::Dependency(dependency_id) => self.dependencies.remove(dependency_id),
    }
  }

  pub fn is_empty(&self) -> bool {
    self.modules.is_empty() && self.dependencies.is_empty()
  }

  pub fn modules(&self) -> &IdentifierSet {
    &self.modules
  }

  pub fn dependencies(&self) -> &UkeySet<DependencyId> {
    &self.dependencies
  }
}

/// Used to count file usage and track which modules/dependencies use each file
#[derive(Debug, Default)]
pub struct FileCounter {
  inner: ArcPathMap<PathResourceIds>,
  incremental_info: IncrementalInfo<ArcPath, BuildHasherDefault<IdentityHasher>>,
}

impl FileCounter {
  /// Add batch [`PathBuf`] to counter
  ///
  /// It will add resource_id at the PathBuf in inner hashmap
  pub fn add_files(&mut self, resource_id: &ResourceId, paths: &ArcPathSet) {
    for path in paths {
      let list = self.inner.entry(path.clone()).or_default();
      if list.is_empty() {
        self.incremental_info.mark_as_add(path);
      }
      // multiple additions are allowed without additional checks to see if the addition was successful
      list.insert(resource_id);
    }
  }

  /// Remove batch [`PathBuf`] from counter
  ///
  /// It will remove resource_id at the PathBuf in inner hashmap
  ///
  /// If the PathBuf resource_id is empty after reduction, the record will be deleted
  /// If PathBuf does not exist, panic will occur.
  pub fn remove_files(&mut self, resource_id: &ResourceId, paths: &ArcPathSet) {
    for path in paths {
      let Some(list) = self.inner.get_mut(path) else {
        panic!("unable to remove untracked file {}", path.to_string_lossy());
      };
      if !list.remove(resource_id) {
        panic!(
          "unable to remove path '{}' with resource_id '{:?}', it has not been added.",
          path.to_string_lossy(),
          resource_id,
        )
      }
      if list.is_empty() {
        self.incremental_info.mark_as_remove(path);
        self.inner.remove(path);
      }
    }
  }

  /// Get the file that has been used
  pub fn files(&self) -> impl Iterator<Item = &ArcPath> {
    self.inner.keys()
  }

  /// Get the resource ids (modules/dependencies) that use a specific file
  pub fn related_resource_ids(&self, path: &ArcPath) -> Option<&PathResourceIds> {
    self.inner.get(path)
  }

  /// reset incremental info
  pub fn reset_incremental_info(&mut self) {
    self.incremental_info.reset();
  }

  /// Added files compared to the `files()` when call reset_incremental_info
  pub fn added_files(&self) -> impl Iterator<Item = &ArcPath> {
    self.incremental_info.added().iter()
  }

  /// Updated files compared to the `files()` when call reset_incremental_info
  pub fn updated_files(&self) -> impl Iterator<Item = &ArcPath> {
    self.incremental_info.updated().iter()
  }

  /// Removed files compared to the `files()` when call reset_incremental_info
  pub fn removed_files(&self) -> impl Iterator<Item = &ArcPath> {
    self.incremental_info.removed().iter()
  }
}

#[cfg(test)]
mod test {
  use rspack_paths::{ArcPath, ArcPathSet};

  use super::{FileCounter, ResourceId};
  use crate::DependencyId;
  #[test]
  fn file_counter_is_available() {
    let mut counter = FileCounter::default();
    let file_a = ArcPath::from(std::path::PathBuf::from("/a"));
    let file_b = ArcPath::from(std::path::PathBuf::from("/b"));
    let file_list_a = {
      let mut list = ArcPathSet::default();
      list.insert(file_a.clone());
      list
    };
    let file_list_b = {
      let mut list = ArcPathSet::default();
      list.insert(file_b.clone());
      list
    };
    let file_list_all = {
      let mut list = ArcPathSet::default();
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
    let file_a = ArcPath::from(std::path::PathBuf::from("/a"));
    let file_list_a = {
      let mut list = ArcPathSet::default();
      list.insert(file_a);
      list
    };
    let resource = ResourceId::Module("A".into());
    counter.remove_files(&resource, &file_list_a);
  }

  #[test]
  fn file_counter_reset_incremental_info() {
    let mut counter = FileCounter::default();
    let file_a = ArcPath::from(std::path::PathBuf::from("/a"));
    let file_list_a = {
      let mut list = ArcPathSet::default();
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
    let file = ArcPath::from(std::path::PathBuf::from("/a"));
    let file_list = {
      let mut list = ArcPathSet::default();
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
