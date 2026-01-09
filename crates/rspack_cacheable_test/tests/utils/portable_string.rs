use std::path::{Path, PathBuf};

use rspack_cacheable::{
  CacheableContext, cacheable,
  utils::PortableString,
  with::{As, AsVec},
};

/// Test context with project_root
struct TestContext(Option<PathBuf>);

impl CacheableContext for TestContext {
  fn project_root(&self) -> Option<&Path> {
    self.0.as_deref()
  }
}

/// Test struct with string identifiers containing paths
#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Data {
  #[cacheable(with=AsVec<As<PortableString>>)]
  paths: Vec<String>,
}

#[test]
#[cfg(not(windows))]
fn test_basic_feature() {
  let context = TestContext(Some(PathBuf::from("/home/user/project")));

  let data = Data {
    paths: vec![
      "ignored|/home/user/project/src/".into(),
      "ignored|/home/user/project/src/a.js".into(),
      "home/home/user/project/src/".into(),
      "/home/user/project/src/".into(),
      "/home/user/project/a.js".into(),
      "/home/user/project/src/a.js".into(),
      "ignored|/home/user/project/a/ other|/home/user/project/b/c.js".into(),
      "ignored|/home/user/aaa/?name=/home/user/bbb/d.txt".into(),
      "javascript/auto|/home/user/project/src/|layer1".into(),
      "just a regular string without any paths".into(),
      "/home/user/project/src/文本.txt".into(),
      "ignored|./a/b/c/d?query=./e/f/g".into(),
      "/home/user/project/src/loader.js!/home/user/project/index.js".into(),
      "/home/user".into(),
      "/home".into(),
      "/root".into(),
    ],
  };

  let bytes = rspack_cacheable::to_bytes(&data, &context).unwrap();

  // Deserialize with different project_root
  let new_context = TestContext(Some(PathBuf::from("/workspace")));
  let new_data: Data = rspack_cacheable::from_bytes(&bytes, &new_context).unwrap();

  assert_eq!(
    new_data.paths,
    vec![
      "ignored|/workspace/src/",
      "ignored|/workspace/src/a.js",
      "home/home/user/project/src/",
      "/workspace/src/",
      "/workspace/a.js",
      "/workspace/src/a.js",
      "ignored|/workspace/a/ other|/workspace/b/c.js",
      "ignored|/aaa/?name=/bbb/d.txt",
      "javascript/auto|/workspace/src/|layer1",
      "just a regular string without any paths",
      "/workspace/src/文本.txt",
      "ignored|./a/b/c/d?query=./e/f/g",
      "/workspace/src/loader.js!/workspace/index.js",
      "/user",
      "/home",
      "/root"
    ]
  );
}

#[test]
#[cfg(not(windows))]
fn test_no_project_root() {
  let context = TestContext(None);

  let data = Data {
    paths: vec!["ignored|/home/user/project/src/main.rs".into()],
  };

  let bytes = rspack_cacheable::to_bytes(&data, &context).unwrap();
  let new_data: Data = rspack_cacheable::from_bytes(&bytes, &context).unwrap();

  // Without project_root, paths should remain unchanged
  assert_eq!(new_data.paths, data.paths);
}

#[test]
#[cfg(windows)]
fn test_windows() {
  let context = TestContext(Some(PathBuf::from("C:\\Users\\project")));

  let data = Data {
    paths: vec![
      "ignored|C:\\Users\\project\\src\\".into(),
      "ignored|C:\\Users\\project\\src\\a.js".into(),
      "cC:\\Users\\project\\src\\".into(),
      "C:\\Users\\project\\src\\".into(),
      "C:\\Users\\project\\a.js".into(),
      "C:\\Users\\project\\src\\a.js".into(),
      "ignored|C:\\Users\\project\\a\\ other|C:\\Users\\project\\b\\c.js".into(),
      "ignored|C:\\Users\\aaa\\?name=C:\\Users\\bbb\\d.txt".into(),
      "javascript/auto|C:\\Users\\project\\src\\|layer1".into(),
      "just a regular string without any paths".into(),
      "C:\\Users\\project\\src\\文本.txt".into(),
      "ignored|.\\a\\b\\c\\d?query=.\\e\\f\\g".into(),
      "C:\\Users\\project\\src\\loader.js!C:\\Users\\project\\index.js".into(),
      "C:\\Users".into(),
      "C:\\".into(),
      "C:\\Windows".into(),
    ],
  };

  let bytes = rspack_cacheable::to_bytes(&data, &context).unwrap();

  let new_context = TestContext(Some(PathBuf::from("D:\\other")));
  let new_data: Data = rspack_cacheable::from_bytes(&bytes, &new_context).unwrap();

  assert_eq!(
    new_data.paths,
    vec![
      "ignored|D:\\other\\src\\",
      "ignored|D:\\other\\src\\a.js",
      "cC:\\Users\\project\\src\\",
      "D:\\other\\src\\",
      "D:\\other\\a.js",
      "D:\\other\\src\\a.js",
      "ignored|D:\\other\\a\\ other|D:\\other\\b\\c.js",
      "ignored|D:\\aaa\\?name=D:\\bbb\\d.txt",
      "javascript/auto|D:\\other\\src\\|layer1",
      "just a regular string without any paths",
      "D:\\other\\src\\文本.txt",
      "ignored|.\\a\\b\\c\\d?query=.\\e\\f\\g",
      "D:\\other\\src\\loader.js!D:\\other\\index.js",
      "D:\\Users".into(),
      "D:\\".into(),
      "D:\\Windows".into(),
    ]
  );
}
