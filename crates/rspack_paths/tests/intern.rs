use std::{
  hash::{BuildHasherDefault, Hash, Hasher},
  path::{Path, PathBuf},
};

use rspack_paths::{IdentityHasher, InternedPath, InternedPathSet, hash_path};

/// Whether both handles point at the same interned allocation.
fn same_allocation(a: &InternedPath, b: &InternedPath) -> bool {
  std::ptr::eq(a.as_ref() as *const Path, b.as_ref() as *const Path)
}

#[test]
fn equal_paths_share_one_allocation() {
  let a = InternedPath::from(Path::new("/intern/a.js"));
  let b = InternedPath::from(PathBuf::from("/intern/a.js"));
  let c = InternedPath::from(Path::new("/intern/b.js"));

  assert_eq!(a, b);
  assert!(same_allocation(&a, &b));
  assert_ne!(a, c);
  assert_eq!(a.as_ref(), Path::new("/intern/a.js"));
}

#[test]
fn hashes_by_content_not_by_pointer() {
  // `InternedPathMap`/`InternedPathSet` feed this hash straight into `IdentityHasher`, so it has to stay
  // the precomputed content hash rather than the interned pointer address.
  let path = Path::new("/intern/hash.js");
  let mut hasher = IdentityHasher::default();
  InternedPath::from(path).hash(&mut hasher);

  assert_eq!(hasher.finish(), hash_path(path));
}

#[test]
fn identity_hashed_set_still_finds_entries() {
  let mut set: InternedPathSet = InternedPathSet::with_hasher(BuildHasherDefault::default());
  set.insert(InternedPath::from(Path::new("/intern/set.js")));

  assert!(set.contains(&InternedPath::from(PathBuf::from("/intern/set.js"))));
  assert!(!set.contains(&InternedPath::from(Path::new("/intern/absent.js"))));
}

#[test]
fn from_parts_interns_into_the_same_object() {
  // `from_parts` is how `rspack_resolver` hands over a path it has already hashed. Feeding the
  // precomputed hash has to land in the same shard as hashing here, or deduplication silently
  // stops working.
  let path = Path::new("/intern/from_parts.js");
  let hashed_here = InternedPath::from(path);
  let hashed_upstream = InternedPath::from_parts(hash_path(path), path);

  assert!(same_allocation(&hashed_here, &hashed_upstream));
}

#[test]
fn reinterning_a_freed_path_still_dedupes() {
  let path = Path::new("/intern/freed.js");
  drop(InternedPath::from(path));

  let a = InternedPath::from(path);
  let b = InternedPath::from(path);
  assert!(same_allocation(&a, &b));
}
