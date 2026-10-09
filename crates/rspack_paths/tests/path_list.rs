//! Tests for shared path lists: content equality, ordering, lifetime and the
//! cacheable converter.

use rspack_paths::{InternedPath, SharedPathList};
use rustc_hash::FxHashSet;

/// Whether both lists point at the same element buffer.
fn same_allocation(a: &SharedPathList, b: &SharedPathList) -> bool {
  std::ptr::eq(a.as_slice().as_ptr(), b.as_slice().as_ptr())
}

#[test]
fn equal_lists_can_be_shared_by_an_owner() {
  let a: InternedPath = "/a".into();
  let b: InternedPath = "/b".into();
  let l1 = SharedPathList::new(&[a.clone(), b.clone()]);
  let l2 = SharedPathList::from_vec(vec![a.clone(), b.clone()]);
  let l3 = SharedPathList::new(&[b, a]);
  assert_eq!(l1, l2);
  assert!(!same_allocation(&l1, &l2), "construction does not intern");
  let mut owner = FxHashSet::default();
  owner.insert(l1.clone());
  let shared = owner.get(&l2).unwrap().clone();
  assert!(!owner.contains(&l3), "element order is part of the key");
  assert!(same_allocation(&l1, &shared));
  assert_eq!(l1.as_slice(), shared.as_slice());
  assert_eq!(l1.len(), 2);
  // dropping one handle keeps the other valid
  drop(l1);
  assert_eq!(shared.as_slice().len(), 2);
}

#[test]
fn empty_lists_need_no_allocation() {
  let e1 = SharedPathList::new(&[]);
  let e2 = SharedPathList::from_vec(Vec::new());
  assert!(e1.is_empty());
  assert_eq!(e1, SharedPathList::default());
  assert!(same_allocation(&e1, &e2));
}

/// The crate depends on rspack_cacheable only through the optional, default-on
/// cacheable feature, so the test target has to be gated the same way.
#[cfg(feature = "cacheable")]
#[test]
fn cacheable_roundtrip() {
  let paths: Vec<InternedPath> = (0..20)
    .map(|i| InternedPath::from(format!("/p/{i}").as_str()))
    .collect();
  let list = SharedPathList::new(&paths);
  let bytes = rspack_cacheable::to_bytes(&list, &()).unwrap();
  let restored: SharedPathList = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(restored.as_slice(), list.as_slice());
  assert!(
    !same_allocation(&list, &restored),
    "decoding leaves deduplication to the owner"
  );
  let mut owner = FxHashSet::default();
  owner.insert(list.clone());
  assert!(same_allocation(&list, owner.get(&restored).unwrap()));
}

#[test]
fn shared_lists_outlive_their_owner() {
  let first: SharedPathList = ["/dead/a", "/dead/b"]
    .into_iter()
    .map(InternedPath::from)
    .collect();
  let second: SharedPathList = ["/dead/a", "/dead/b"]
    .into_iter()
    .map(InternedPath::from)
    .collect();
  let mut owner = FxHashSet::default();
  owner.insert(first);
  let shared = owner.get(&second).unwrap().clone();
  let ptr = shared.as_slice().as_ptr();
  drop(owner);
  assert_eq!(shared, second);
  assert_eq!(shared.as_slice().as_ptr(), ptr);
  let clone = shared.clone();
  drop(shared);
  assert_eq!(clone, second);
  assert_eq!(clone.as_slice().as_ptr(), ptr);
}
