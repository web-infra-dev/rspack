use std::{
  path::{Path, PathBuf},
  ptr,
};

use rspack_cacheable::{
  CacheableContext, Error, from_bytes,
  rkyv::{Archived, access, util::AlignedVec},
  to_bytes,
};
use rspack_paths::{InternedPath, InternedPathSet};

struct TestContext(Option<PathBuf>);

impl CacheableContext for TestContext {
  fn project_root(&self) -> Option<&Path> {
    self.0.as_deref()
  }
}

fn project_roots() -> (PathBuf, PathBuf) {
  #[cfg(windows)]
  {
    (
      PathBuf::from(r"C:\old\project"),
      PathBuf::from(r"D:\new\project"),
    )
  }
  #[cfg(not(windows))]
  {
    (PathBuf::from("/old/project"), PathBuf::from("/new/project"))
  }
}

#[test]
fn test_interned_path_portable_relocates_project_root() {
  let (old_root, new_root) = project_roots();
  let paths = vec![
    InternedPath::from(old_root.join("src").join("main.js")),
    InternedPath::from(old_root.join("node_modules").join("pkg").join("index.js")),
    InternedPath::from(&old_root),
  ];

  let bytes = to_bytes(&paths, &TestContext(Some(old_root))).unwrap();
  let restored: Vec<InternedPath> =
    from_bytes(&bytes, &TestContext(Some(new_root.clone()))).unwrap();

  assert_eq!(
    restored,
    vec![
      InternedPath::from(new_root.join("src").join("main.js")),
      InternedPath::from(new_root.join("node_modules").join("pkg").join("index.js")),
      InternedPath::from(new_root),
    ]
  );
}

#[test]
fn test_interned_path_portable_set_relocates_project_root() {
  let (old_root, new_root) = project_roots();
  let paths: InternedPathSet = ["package.json", "build.config.json"]
    .into_iter()
    .map(|file| InternedPath::from(old_root.join(file)))
    .collect();
  let bytes = to_bytes(&paths, &TestContext(Some(old_root))).unwrap();
  let restored: InternedPathSet = from_bytes(&bytes, &TestContext(Some(new_root.clone()))).unwrap();

  assert_eq!(restored.len(), 2);
  for file in ["package.json", "build.config.json"] {
    assert!(restored.contains(&InternedPath::from(new_root.join(file))));
  }
}

#[test]
fn test_interned_path_portable_preserves_relative_and_empty_paths() {
  let (old_root, new_root) = project_roots();
  let paths = vec![
    InternedPath::from("src/main.js"),
    InternedPath::from("../shared/index.js"),
    InternedPath::from(""),
  ];

  let bytes = to_bytes(&paths, &TestContext(Some(old_root))).unwrap();
  let restored: Vec<InternedPath> = from_bytes(&bytes, &TestContext(Some(new_root))).unwrap();

  assert_eq!(restored, paths);
}

#[test]
fn test_interned_path_native_roundtrip_preserves_path_bytes() {
  let (root, _) = project_roots();
  let paths = vec![
    InternedPath::from(root.join("src").join("main.js")),
    InternedPath::from("./src//main.js"),
    InternedPath::from("../shared/"),
    InternedPath::from(""),
  ];
  let context = TestContext(None);

  let bytes = to_bytes(&paths, &context).unwrap();
  let restored: Vec<InternedPath> = from_bytes(&bytes, &context).unwrap();

  assert_eq!(restored, paths);
  for (actual, expected) in restored.iter().zip(&paths) {
    assert_eq!(
      actual.as_path().as_os_str().as_encoded_bytes(),
      expected.as_path().as_os_str().as_encoded_bytes()
    );
  }
}

#[test]
fn test_interned_path_archive_shares_repeated_and_empty_paths() {
  let (root, _) = project_roots();
  let path = root.join("src").join("shared.js");
  let paths = vec![
    InternedPath::from(&path),
    InternedPath::from(&path),
    InternedPath::from(""),
    InternedPath::from(""),
  ];

  for context in [TestContext(None), TestContext(Some(root))] {
    let bytes = to_bytes(&paths, &context).unwrap();
    // to_bytes returns a Vec<u8>, whose alignment is not guaranteed for archived access.
    let mut aligned = AlignedVec::<16>::new();
    aligned.extend_from_slice(&bytes);
    let archived = access::<Archived<Vec<InternedPath>>, Error>(&aligned).unwrap();

    assert_eq!(archived.len(), paths.len());
    assert!(ptr::eq(archived[0].get(), archived[1].get()));
    assert!(ptr::eq(archived[2].get(), archived[3].get()));
    assert!(!ptr::eq(archived[0].get(), archived[2].get()));
  }
}

#[test]
#[cfg(unix)]
fn test_interned_path_native_preserves_non_utf8_bytes() {
  use std::{ffi::OsString, os::unix::ffi::OsStringExt};

  let path = PathBuf::from(OsString::from_vec(
    b"/project/src/invalid-\xff-\xfe.js".to_vec(),
  ));
  let original = InternedPath::from(&path);
  let context = TestContext(None);

  let bytes = to_bytes(&original, &context).unwrap();
  let restored: InternedPath = from_bytes(&bytes, &context).unwrap();

  assert_eq!(
    restored.as_path().as_os_str().as_encoded_bytes(),
    path.as_os_str().as_encoded_bytes()
  );
  assert_eq!(restored, original);
}
