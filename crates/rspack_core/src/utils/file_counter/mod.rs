mod incremental_info;
mod resource_id;

use incremental_info::IncrementalInfo;
use rspack_paths::{ArcPath, ArcPathMap, ArcPathSet};
use rustc_hash::FxHashSet as HashSet;

pub use self::resource_id::ResourceId;

/// Used to count file usage and track which modules/dependencies use each file
#[derive(Debug, Default)]
pub struct FileCounter {
  inner: ArcPathMap<HashSet<ResourceId>>,
  incremental_info: IncrementalInfo,
}

impl FileCounter {
  /// Add batch [`PathBuf`] to counter
  ///
  /// It will add resource_id at the PathBuf in inner hashmap
  pub fn add_files(&mut self, resource_id: &ResourceId, paths: &ArcPathSet) {
    for path in paths {
      // Use get_mut instead of entry API to avoid unnecessary path cloning,
      // since paths are frequently duplicated in practice
      if let Some(resources) = self.inner.get_mut(path) {
        resources.insert(resource_id.clone());
      } else {
        let mut resources = HashSet::default();
        resources.insert(resource_id.clone());
        self.inner.insert(path.clone(), resources);
        self.incremental_info.add(path);
      }
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
        self.incremental_info.remove(path);
        self.inner.remove(path);
      }
    }
  }

  /// Get the file that has been used
  pub fn files(&self) -> impl Iterator<Item = &ArcPath> {
    self.inner.keys()
  }

  /// Get the resource ids (modules/dependencies) that use a specific file
  pub fn related_resource_ids(&self, path: &ArcPath) -> Option<&HashSet<ResourceId>> {
    self.inner.get(path)
  }

  /// reset incremental info
  pub fn reset_incremental_info(&mut self) {
    self.incremental_info.reset();
  }

  /// Added files compared to the `files()` when call reset_incremental_info
  pub fn added_files(&self) -> &ArcPathSet {
    self.incremental_info.added_files()
  }

  /// Removed files compared to the `files()` when call reset_incremental_info
  pub fn removed_files(&self) -> &ArcPathSet {
    self.incremental_info.removed_files()
  }
}

#[cfg(test)]
mod test {
  use rspack_paths::{ArcPath, ArcPathSet};

  use super::{FileCounter, ResourceId};
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
    assert_eq!(counter.files().collect::<Vec<_>>().len(), 2);
    assert_eq!(counter.added_files().len(), 2);
    assert_eq!(counter.removed_files().len(), 0);

    // test repeated additions
    counter.add_files(&resource_1, &file_list_all);
    assert_eq!(counter.files().collect::<Vec<_>>().len(), 2);
    assert_eq!(counter.added_files().len(), 2);
    assert_eq!(counter.removed_files().len(), 0);

    counter.remove_files(&resource_1, &file_list_a);
    assert_eq!(counter.files().collect::<Vec<_>>().len(), 2);
    assert_eq!(counter.added_files().len(), 2);
    assert_eq!(counter.removed_files().len(), 0);

    counter.remove_files(&resource_1, &file_list_b);
    assert_eq!(counter.files().collect::<Vec<_>>().len(), 1);
    assert_eq!(counter.added_files().len(), 1);
    assert_eq!(counter.removed_files().len(), 0);

    counter.remove_files(&resource_2, &file_list_a);
    assert_eq!(counter.files().collect::<Vec<_>>().len(), 0);
    assert_eq!(counter.added_files().len(), 0);
    assert_eq!(counter.removed_files().len(), 0);
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
    assert_eq!(counter.added_files().len(), 1);
    assert_eq!(counter.removed_files().len(), 0);

    counter.reset_incremental_info();
    assert_eq!(counter.added_files().len(), 0);
    assert_eq!(counter.removed_files().len(), 0);

    counter.add_files(&resource_2, &file_list_a);
    assert_eq!(counter.added_files().len(), 0);
    assert_eq!(counter.removed_files().len(), 0);

    counter.remove_files(&resource_1, &file_list_a);
    assert_eq!(counter.added_files().len(), 0);
    assert_eq!(counter.removed_files().len(), 0);

    counter.remove_files(&resource_2, &file_list_a);
    assert_eq!(counter.added_files().len(), 0);
    assert_eq!(counter.removed_files().len(), 1);
  }
}
