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
#[cfg(not(windows))]
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

  // Test path outside project_root (now uses relative paths with ..)
  let outside_path = PathData {
    path1: PathBuf::from("/usr/lib/a.so"),
    path2: Utf8PathBuf::from("/usr/lib/src/b.txt"),
  };
  let bytes = rspack_cacheable::to_bytes(&outside_path, &context).unwrap();
  let new_data: PathData = rspack_cacheable::from_bytes(&bytes, &new_context).unwrap();
  // The paths are now relative to project_root, so they get translated
  // /usr/lib/a.so relative to /home/user/project is ../../../usr/lib/a.so
  // When deserialized with /new/location as project_root:
  // /new/location/../../../usr/lib/a.so => /usr/lib/a.so (after normalization)
  assert_eq!(new_data, outside_path);
}

#[test]
#[cfg(not(windows))]
fn test_relative_path_outside_project() {
  // Test that paths outside project_root are converted to relative paths with ".."
  let context = TestContext(Some(PathBuf::from("/home/user/project")));

  let data = PathData {
    // /home/other/file.txt relative to /home/user/project => ../../other/file.txt
    path1: PathBuf::from("/home/other/file.txt"),
    // /etc/config.ini relative to /home/user/project => ../../../etc/config.ini
    path2: Utf8PathBuf::from("/etc/config.ini"),
  };

  let bytes = rspack_cacheable::to_bytes(&data, &context).unwrap();

  // Deserialize with different project_root
  let new_context = TestContext(Some(PathBuf::from("/workspace/app")));
  let new_data: PathData = rspack_cacheable::from_bytes(&bytes, &new_context).unwrap();

  // Paths maintain their relative structure from project_root:
  // ../../other/file.txt from /workspace/app => /workspace/other/file.txt
  // But due to normalization: /workspace/app/../../other/file.txt => /other/file.txt
  assert_eq!(new_data.path1, PathBuf::from("/other/file.txt"));
  // ../../../etc/config.ini from /workspace/app => /etc/config.ini
  assert_eq!(new_data.path2, PathBuf::from("/etc/config.ini"));
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

/// Paths are stored slash-separated, so restoring one has to put the separators back into the
/// platform's own form. The assertions compare strings, not `Path`s: `Path` equality on Windows
/// compares components, so it sees `D:/a/b` and `D:\a\b` as the same path while the consumers
/// that broke in https://github.com/web-infra-dev/rspack/issues/15352 (watchpack) compare the
/// spelling.
#[test]
fn test_round_trip_keeps_native_separator() {
  let project_root = PathBuf::from("a").join("project");
  let path = project_root.join("src").join("main.rs");

  // Without a project root the path is stored as-is and restored verbatim.
  let restored = PortablePath::new(&path, None).into_path_string(None);
  assert_eq!(restored, path.to_string_lossy());

  // With one it round trips through a relative path.
  let restored =
    PortablePath::new(&path, Some(&project_root)).into_path_string(Some(&project_root));
  assert_eq!(restored, path.to_string_lossy());
}

#[test]
#[cfg(windows)]
fn test_absolute_round_trip_keeps_native_separator() {
  let project_root = PathBuf::from("D:\\workspace\\app");
  let path = PathBuf::from("D:\\workspace\\app\\src\\main.rs");

  let restored = PortablePath::new(&path, None).into_path_string(None);
  assert_eq!(restored, "D:\\workspace\\app\\src\\main.rs");

  let restored =
    PortablePath::new(&path, Some(&project_root)).into_path_string(Some(&project_root));
  assert_eq!(restored, "D:\\workspace\\app\\src\\main.rs");

  // A path outside the project root travels as `..` segments and must come back native too.
  let outside = PathBuf::from("D:\\workspace\\other\\lib.rs");
  let restored =
    PortablePath::new(&outside, Some(&project_root)).into_path_string(Some(&project_root));
  assert_eq!(restored, "D:\\workspace\\other\\lib.rs");
}
