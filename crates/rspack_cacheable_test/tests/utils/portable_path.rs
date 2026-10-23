use std::path::{Path, PathBuf};

use camino::Utf8PathBuf;
use rspack_cacheable::{CacheableContext, cacheable, utils::PortablePath, with::As};

struct TestContext(Option<PathBuf>);

impl CacheableContext for TestContext {
  fn project_root(&self) -> Option<&Path> {
    self.0.as_deref()
  }
}

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct PathData {
  #[cacheable(with=As<PortablePath>)]
  path1: PathBuf,
  #[cacheable(with=As<PortablePath>)]
  path2: Utf8PathBuf,
}

#[test]
#[cfg(not(windows))]
fn test_with_context() {
  let context = TestContext(Some(PathBuf::from("/home/user/project")));
  let new_context = TestContext(Some(PathBuf::from("/new/location")));

  let inside_path = PathData {
    path1: PathBuf::from("/home/user/project/src/main.rs"),
    path2: Utf8PathBuf::from("/home/user/project/src/lib.rs"),
  };

  let bytes = rspack_cacheable::to_bytes(&inside_path, &context).unwrap();
  let new_data: PathData = rspack_cacheable::from_bytes(&bytes, &new_context).unwrap();
  assert_eq!(new_data.path1, PathBuf::from("/new/location/src/main.rs"));
  assert_eq!(new_data.path2, PathBuf::from("/new/location/src/lib.rs"));

  let outside_path = PathData {
    path1: PathBuf::from("/usr/lib/a.so"),
    path2: Utf8PathBuf::from("/usr/lib/src/b.txt"),
  };
  let bytes = rspack_cacheable::to_bytes(&outside_path, &context).unwrap();
  let new_data: PathData = rspack_cacheable::from_bytes(&bytes, &new_context).unwrap();
  assert_eq!(new_data, outside_path);
}

#[test]
#[cfg(not(windows))]
fn test_path_outside_project_is_unchanged() {
  let context = TestContext(Some(PathBuf::from("/home/user/project")));

  let data = PathData {
    path1: PathBuf::from("/home/other/file.txt"),
    path2: Utf8PathBuf::from("/home/user/project-other/config.ini"),
  };

  let bytes = rspack_cacheable::to_bytes(&data, &context).unwrap();

  let new_context = TestContext(Some(PathBuf::from("/workspace/deeper/new/app")));
  let original: PathData = rspack_cacheable::from_bytes(&bytes, &context).unwrap();
  assert_eq!(original, data);
  let new_data: PathData = rspack_cacheable::from_bytes(&bytes, &new_context).unwrap();

  assert_eq!(new_data.path1, PathBuf::from("/home/other/file.txt"));
  assert_eq!(
    new_data.path2,
    PathBuf::from("/home/user/project-other/config.ini")
  );
}

#[test]
fn test_root_binding_is_reset_for_each_encode() {
  let root = std::env::temp_dir().join("rspack-binding-project");
  let context = TestContext(Some(root.clone()));
  let inside = PathData {
    path1: root.join("src/main.rs"),
    path2: Utf8PathBuf::from_path_buf(root.join("src/lib.rs")).unwrap(),
  };
  let outside = PathData {
    path1: root.parent().unwrap().join("outside.rs"),
    path2: Utf8PathBuf::from_path_buf(root.join("src/lib.rs")).unwrap(),
  };

  let (_, bound) = rspack_cacheable::to_bytes_with_binding(&outside, &context).unwrap();
  assert!(bound);
  let (_, bound) = rspack_cacheable::to_bytes_with_binding(&inside, &context).unwrap();
  assert!(!bound);
  let (_, bound) = rspack_cacheable::to_bytes_with_binding(&outside, &TestContext(None)).unwrap();
  assert!(!bound);
}

#[test]
#[cfg(windows)]
fn test_windows_path() {
  let context = TestContext(Some(PathBuf::from("C:\\Users\\test\\project")));

  let data = PathData {
    path1: PathBuf::from("C:\\Users\\test\\project\\src\\main.rs"),
    path2: Utf8PathBuf::from("C:\\Users\\test\\project\\src\\lib.rs"),
  };

  let bytes = rspack_cacheable::to_bytes(&data, &context).unwrap();

  let new_context = TestContext(Some(PathBuf::from("D:\\workspace")));
  let new_data: PathData = rspack_cacheable::from_bytes(&bytes, &new_context).unwrap();

  assert_eq!(new_data.path1, PathBuf::from("D:\\workspace\\src\\main.rs"));
  assert_eq!(new_data.path2, PathBuf::from("D:\\workspace\\src\\lib.rs"));

  let outside = PathData {
    path1: PathBuf::from(r"C:\Users\external\main.rs"),
    path2: Utf8PathBuf::from(r"C:\Users\test\project-other\lib.rs"),
  };
  let bytes = rspack_cacheable::to_bytes(&outside, &context).unwrap();
  for root in [&context, &new_context] {
    let restored: PathData = rspack_cacheable::from_bytes(&bytes, root).unwrap();
    assert_eq!(
      restored.path1.to_string_lossy(),
      outside.path1.to_string_lossy()
    );
    assert_eq!(restored.path2.as_str(), outside.path2.as_str());
  }
}

#[test]
#[cfg(windows)]
fn test_windows_path_without_context_uses_native_separators() {
  let context = TestContext(None);
  let data = PathData {
    path1: PathBuf::from("C:\\Users\\test\\project\\src\\main.rs"),
    path2: Utf8PathBuf::from("C:\\Users\\test\\project\\src\\lib.rs"),
  };

  let bytes = rspack_cacheable::to_bytes(&data, &context).unwrap();
  let new_data: PathData = rspack_cacheable::from_bytes(&bytes, &context).unwrap();

  assert_eq!(
    new_data.path1.to_string_lossy(),
    "C:\\Users\\test\\project\\src\\main.rs"
  );
  assert_eq!(
    new_data.path2.as_str(),
    "C:\\Users\\test\\project\\src\\lib.rs"
  );
}

#[test]
#[cfg(windows)]
fn test_windows_relative_paths_preserve_their_representation() {
  let context = TestContext(None);
  let data = PathData {
    path1: PathBuf::from("./build.txt"),
    path2: Utf8PathBuf::from("node_modules/"),
  };

  let bytes = rspack_cacheable::to_bytes(&data, &context).unwrap();
  let new_data: PathData = rspack_cacheable::from_bytes(&bytes, &context).unwrap();

  assert_eq!(new_data.path1.to_string_lossy(), "./build.txt");
  assert_eq!(new_data.path2.as_str(), "node_modules/");
}

struct AliasedContext {
  root: PathBuf,
  canonical_root: PathBuf,
}

impl CacheableContext for AliasedContext {
  fn project_root(&self) -> Option<&Path> {
    Some(&self.root)
  }

  fn canonical_project_root(&self) -> Option<&Path> {
    Some(&self.canonical_root)
  }
}

#[test]
fn test_aliased_root_restores_each_spelling() {
  let temp = std::env::temp_dir();
  let writer = AliasedContext {
    root: temp.join("link-a").join("project"),
    canonical_root: temp.join("real-a").join("project"),
  };
  let reader = AliasedContext {
    root: temp.join("link-b").join("deeper").join("project"),
    canonical_root: temp.join("real-b").join("project"),
  };
  let paths = |context: &AliasedContext| PathData {
    path1: context.canonical_root.join("src").join("main.rs"),
    path2: Utf8PathBuf::from_path_buf(context.root.clone()).unwrap(),
  };

  let (bytes, bound) = rspack_cacheable::to_bytes_with_binding(&paths(&writer), &writer).unwrap();
  assert!(!bound);
  let decoded: PathData = rspack_cacheable::from_bytes(&bytes, &reader).unwrap();
  assert_eq!(decoded, paths(&reader));

  let outside = PathData {
    path1: temp.join("real-a").join("other.rs"),
    path2: Utf8PathBuf::from_path_buf(writer.root.join("lib.rs")).unwrap(),
  };
  let (bytes, bound) = rspack_cacheable::to_bytes_with_binding(&outside, &writer).unwrap();
  assert!(bound);
  let decoded: PathData = rspack_cacheable::from_bytes(&bytes, &reader).unwrap();
  assert_eq!(decoded.path1, outside.path1);
}
