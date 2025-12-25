use std::path::{Path, PathBuf};

use camino::Utf8PathBuf;
use rspack_cacheable::{CacheableContext, cacheable, utils::PortablePath, with::As};

/// Test context with project_root
struct TestContext(Option<PathBuf>);

impl CacheableContext for TestContext {
  fn project_root(&self) -> Option<&Path> {
    self.0.as_deref()
  }
}

/// Test struct with multiple paths
#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct PathData {
  #[cacheable(with=As<PortablePath>)]
  path1: PathBuf,
  #[cacheable(with=As<PortablePath>)]
  path2: Utf8PathBuf,
}

#[test]
fn test_with_context() {
  let context = TestContext(Some(PathBuf::from("/home/user/project")));
  let new_context = TestContext(Some(PathBuf::from("/new/location")));

  // Test path inside project_root
  let inside_path = PathData {
    path1: PathBuf::from("/home/user/project/src/main.rs"),
    path2: Utf8PathBuf::from("/home/user/project/src/lib.rs"),
  };

  let bytes = rspack_cacheable::to_bytes(&inside_path, &context).unwrap();
  let new_data: PathData = rspack_cacheable::from_bytes(&bytes, &new_context).unwrap();
  assert_eq!(new_data.path1, PathBuf::from("/new/location/src/main.rs"));
  assert_eq!(new_data.path2, PathBuf::from("/new/location/src/lib.rs"));

  // Test path outside project_root
  let outside_path = PathData {
    path1: PathBuf::from("/usr/lib/a.so"),
    path2: Utf8PathBuf::from("/usr/lib/src/b.txt"),
  };
  let bytes = rspack_cacheable::to_bytes(&outside_path, &context).unwrap();
  let new_data: PathData = rspack_cacheable::from_bytes(&bytes, &new_context).unwrap();
  assert_eq!(new_data, outside_path);
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

  // Deserialize on different Windows path
  let new_context = TestContext(Some(PathBuf::from("D:\\workspace")));
  let new_data: PathData = rspack_cacheable::from_bytes(&bytes, &new_context).unwrap();

  // Path separators should be normalized
  assert_eq!(new_data.path1, PathBuf::from("D:\\workspace\\src\\main.rs"));
  assert_eq!(new_data.path2, PathBuf::from("D:\\workspace\\src\\lib.rs"));
}
