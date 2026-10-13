//! The file counter's reverse index stores the dependency ids of one interned
//! path list once, no matter how many paths the list mentions.

use rspack_core::{DependencyId, FileCounter, ModuleIdentifier};
use rspack_paths::{InternedPath, InternedPathList};

fn path(value: &str) -> InternedPath {
  InternedPath::from(std::path::PathBuf::from(value))
}

/// The dependency ids the counter reports for one path, sorted.
fn dependencies(counter: &FileCounter, path: &InternedPath) -> Vec<u32> {
  let mut dependencies: Vec<u32> = counter
    .related_resource_ids(path)
    .expect("path should be tracked")
    .dependencies()
    .map(|id| id.as_u32())
    .collect();
  dependencies.sort_unstable();
  dependencies
}

#[test]
fn equal_lists_keep_the_same_membership() {
  let mut counter = FileCounter::default();
  let a = path("/a");
  let b = path("/b");

  // Two lists with equal content share one interned handle, so the ids are
  // stored once and removal finds the entry through either handle.
  let first = InternedPathList::new(&[a.clone(), b.clone()]);
  let second = InternedPathList::new(&[a.clone(), b.clone()]);
  counter.add_dependency_files(DependencyId::from(1), &first);
  counter.add_dependency_files(DependencyId::from(2), &second);
  assert_eq!(dependencies(&counter, &a), vec![1, 2]);
  assert_eq!(dependencies(&counter, &b), vec![1, 2]);

  counter.remove_dependency_files(DependencyId::from(1), &first);
  assert_eq!(dependencies(&counter, &a), vec![2]);
  assert_eq!(dependencies(&counter, &b), vec![2]);

  // The paths stay only while some list mentions them.
  counter.remove_dependency_files(DependencyId::from(2), &second);
  assert!(counter.related_resource_ids(&a).is_none());
  assert!(counter.related_resource_ids(&b).is_none());
  assert_eq!(counter.files().count(), 0);
}

#[test]
fn repeated_adds_of_the_same_dependency_are_ignored() {
  let mut counter = FileCounter::default();
  let a = path("/a");
  let list = InternedPathList::new(std::slice::from_ref(&a));
  let dependency = DependencyId::from(7);

  counter.add_dependency_files(dependency, &list);
  counter.add_dependency_files(dependency, &list);
  assert_eq!(dependencies(&counter, &a), vec![7]);

  counter.remove_dependency_files(dependency, &list);
  assert!(counter.related_resource_ids(&a).is_none());
}

#[test]
fn different_lists_stay_separate() {
  let mut counter = FileCounter::default();
  let a = path("/a");
  let b = path("/b");
  let only_a = InternedPathList::new(std::slice::from_ref(&a));
  let only_b = InternedPathList::new(std::slice::from_ref(&b));

  counter.add_dependency_files(DependencyId::from(1), &only_a);
  counter.add_dependency_files(DependencyId::from(2), &only_b);
  assert_eq!(dependencies(&counter, &a), vec![1]);
  assert_eq!(dependencies(&counter, &b), vec![2]);
}

#[test]
fn modules_and_dependencies_stay_separate() {
  let mut counter = FileCounter::default();
  let a = path("/a");
  let module: ModuleIdentifier = "module".into();
  let list = InternedPathList::new(std::slice::from_ref(&a));

  counter.add_module_files(module, [&a]);
  counter.add_dependency_files(DependencyId::from(1), &list);

  let ids = counter.related_resource_ids(&a).expect("tracked");
  assert_eq!(ids.modules().len(), 1);
  assert_eq!(ids.dependencies().count(), 1);

  counter.remove_module_files(module, [&a]);
  let ids = counter
    .related_resource_ids(&a)
    .expect("kept by the dependency");
  assert!(ids.modules().is_empty());
  assert_eq!(ids.dependencies().count(), 1);
}

#[test]
fn incremental_info_follows_the_grouped_paths() {
  let mut counter = FileCounter::default();
  let a = path("/a");
  let b = path("/b");
  let list = InternedPathList::new(&[a, b]);
  let dependency = DependencyId::from(1);

  counter.add_dependency_files(dependency, &list);
  assert_eq!(counter.added_files().count(), 2);
  counter.reset_incremental_info();

  counter.remove_dependency_files(dependency, &list);
  assert_eq!(counter.removed_files().count(), 2);
}
