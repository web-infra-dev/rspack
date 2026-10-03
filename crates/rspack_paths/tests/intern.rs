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

#[cfg(windows)]
#[test]
fn windows_slash_spelling_never_becomes_canonical() {
  // A `/` spelling interned first (e.g. a JS loader's `addDependency`) must not make the native
  // spelling of the same file come back with `/`: watchpack keys watchers by the exact string.
  let slash = InternedPath::from(Path::new("D:/intern/win/a.js"));
  let native = InternedPath::from(Path::new(r"D:\intern\win\a.js"));

  assert!(same_allocation(&slash, &native));
  assert_eq!(slash.as_ref().as_os_str(), r"D:\intern\win\a.js");
  assert_eq!(native.as_ref().as_os_str(), r"D:\intern\win\a.js");
}

#[cfg(windows)]
#[test]
fn windows_mixed_spelling_from_parts_is_normalized() {
  // The resolver hands over `from_parts` paths; a `/` base directory from JS yields `D:/a\b`.
  let mixed = Path::new(r"D:/intern/win\mixed.js");
  let interned = InternedPath::from_parts(hash_path(mixed), mixed);
  let native = InternedPath::from(Path::new(r"D:\intern\win\mixed.js"));

  assert_eq!(interned.as_ref().as_os_str(), r"D:\intern\win\mixed.js");
  assert!(same_allocation(&interned, &native));
}

#[cfg(windows)]
#[test]
fn windows_redundant_separators_and_dot_components_are_canonicalized() {
  // Every spelling `Path::components` collapses is one path to `Path::eq`, so all of them must
  // intern to the canonical bytes, whichever arrives first.
  let canonical = InternedPath::from(Path::new(r"D:\intern\win\c.js"));
  for spelling in [
    r"D:\intern\\win\c.js",
    r"D:\intern\.\win\c.js",
    r"D:\intern\win\c.js\.",
  ] {
    let interned = InternedPath::from(Path::new(spelling));
    assert_eq!(
      interned.as_ref().as_os_str(),
      r"D:\intern\win\c.js",
      "{spelling}"
    );
    assert!(same_allocation(&interned, &canonical), "{spelling}");
  }
}

#[cfg(windows)]
#[test]
fn windows_trailing_separator_is_kept_but_spelled_natively() {
  // `**/node_modules/**` matches `node_modules/` but not `node_modules`, so the trailing
  // separator must survive; only its spelling changes.
  let interned = InternedPath::from(Path::new("D:/intern/win/node_modules/"));
  assert_eq!(
    interned.as_ref().as_os_str(),
    r"D:\intern\win\node_modules\"
  );
}

#[cfg(windows)]
#[test]
fn windows_already_canonical_paths_keep_their_bytes() {
  // `..` stays (it is not collapsed by `Path::components` and keeps paths distinct), trailing
  // separators stay, a UNC prefix keeps its leading `\\`, the drive letter's case is not touched,
  // and relative paths are never rewritten.
  for spelling in [
    r"D:\intern\win\..\up.js",
    r"D:\intern\win\dir\",
    r"D:\",
    r"\\server\share\intern\unc.js",
    r"d:\intern\win\lower.js",
    "src/index.js",
    "src/",
  ] {
    let interned = InternedPath::from(Path::new(spelling));
    assert_eq!(interned.as_ref().as_os_str(), spelling, "{spelling}");
  }
}

#[cfg(windows)]
#[test]
fn windows_verbatim_paths_keep_their_bytes() {
  // Under `\\?\` a `/` is an ordinary character, so nothing is rewritten.
  let verbatim = Path::new(r"\\?\D:\intern\win\verbatim/file.js");
  let interned = InternedPath::from(verbatim);

  assert_eq!(interned.as_ref().as_os_str(), verbatim.as_os_str());
}

#[cfg(unix)]
#[test]
fn unix_never_rewrites_bytes() {
  let path = Path::new("/intern/unix/back\\slash.js");
  let interned = InternedPath::from(path);

  assert_eq!(interned.as_ref().as_os_str(), path.as_os_str());
}

#[test]
fn reinterning_a_freed_path_still_dedupes() {
  let path = Path::new("/intern/freed.js");
  drop(InternedPath::from(path));

  let a = InternedPath::from(path);
  let b = InternedPath::from(path);
  assert!(same_allocation(&a, &b));
}
