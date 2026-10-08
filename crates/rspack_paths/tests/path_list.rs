//! Tests for content-interned path lists: sharing, ordering, lifetime and the
//! cacheable converter.

use rspack_paths::{InternedPath, InternedPathList};

/// Whether both lists point at the same element buffer.
fn same_allocation(a: &InternedPathList, b: &InternedPathList) -> bool {
  std::ptr::eq(a.as_slice().as_ptr(), b.as_slice().as_ptr())
}

#[test]
fn equal_lists_share_allocation() {
  let a: InternedPath = "/a".into();
  let b: InternedPath = "/b".into();
  let l1 = InternedPathList::new(&[a.clone(), b.clone()]);
  let l2 = InternedPathList::from_vec(vec![a.clone(), b.clone()]);
  let l3 = InternedPathList::new(&[b, a]);
  assert_eq!(l1, l2);
  assert!(
    !same_allocation(&l1, &l3),
    "element order is part of the key"
  );
  assert!(same_allocation(&l1, &l2));
  assert_eq!(l1.as_slice(), l2.as_slice());
  assert_eq!(l1.len(), 2);
  // dropping one handle keeps the other valid
  drop(l1);
  assert_eq!(l2.as_slice().len(), 2);
}

#[test]
fn empty_lists_share_allocation() {
  let e1 = InternedPathList::new(&[]);
  let e2 = InternedPathList::from_vec(Vec::new());
  assert!(same_allocation(&e1, &e2));
}

#[test]
fn cacheable_roundtrip() {
  let paths: Vec<InternedPath> = (0..20)
    .map(|i| InternedPath::from(format!("/p/{i}").as_str()))
    .collect();
  let list = InternedPathList::new(&paths);
  let bytes = rspack_cacheable::to_bytes(&list, &()).unwrap();
  let restored: InternedPathList = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(restored.as_slice(), list.as_slice());
  assert!(
    same_allocation(&list, &restored),
    "restored list re-interns"
  );
}

#[test]
fn recreated_equal_lists_share_after_last_handle_drops() {
  let first: InternedPathList = ["/dead/a", "/dead/b"]
    .into_iter()
    .map(InternedPath::from)
    .collect();
  drop(first);
  let second: InternedPathList = ["/dead/a", "/dead/b"]
    .into_iter()
    .map(InternedPath::from)
    .collect();
  let third = InternedPathList::from_vec(second.as_slice().to_vec());
  assert!(same_allocation(&second, &third));
}
