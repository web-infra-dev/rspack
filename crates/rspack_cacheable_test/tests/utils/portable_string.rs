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
fn test_string_root_binding() {
  let root = std::env::temp_dir().join("rspack-binding-project");
  let context = TestContext(Some(root.clone()));
  let inside = Data {
    paths: vec![root.join("src/main.js").to_string_lossy().into_owned()],
  };
  let outside = Data {
    paths: vec![
      root
        .parent()
        .unwrap()
        .join("outside/main.js")
        .to_string_lossy()
        .into_owned(),
    ],
  };
  let (_, bound) = rspack_cacheable::to_bytes_with_binding(&outside, &context).unwrap();
  assert!(bound);
  let (_, bound) = rspack_cacheable::to_bytes_with_binding(&inside, &context).unwrap();
  assert!(!bound);
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
      "https://x/ /other/path with space/a.js /home/user/projectish/a.js".into(),
      "/home/user/project/path with space/a.js".into(),
    ],
  };

  let bytes = rspack_cacheable::to_bytes(&data, &context).unwrap();
  let original: Data = rspack_cacheable::from_bytes(&bytes, &context).unwrap();
  assert_eq!(original, data);

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
      "ignored|/home/user/aaa/?name=/home/user/bbb/d.txt",
      "javascript/auto|/workspace/src/|layer1",
      "just a regular string without any paths",
      "/workspace/src/文本.txt",
      "ignored|./a/b/c/d?query=./e/f/g",
      "/workspace/src/loader.js!/workspace/index.js",
      "/home/user",
      "/home",
      "/root",
      "https://x/ /other/path with space/a.js /home/user/projectish/a.js",
      "/workspace/path with space/a.js"
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
  let original: Data = rspack_cacheable::from_bytes(&bytes, &context).unwrap();
  assert_eq!(original, data);
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
      r"C:/Users/project/path with space/a.js".into(),
      r"C:\\Users\\project\\src\\a.js".into(),
      "C:\\Users\\project\\a.js|\0root\\\0|\0escape".into(),
      "C:\\\\Users\\\\project\\\\a.js|\0root\\\\\0".into(),
    ],
  };

  let bytes = rspack_cacheable::to_bytes(&data, &context).unwrap();
  let original: Data = rspack_cacheable::from_bytes(&bytes, &context).unwrap();
  assert_eq!(original, data);

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
      "ignored|C:\\Users\\aaa\\?name=C:\\Users\\bbb\\d.txt",
      "javascript/auto|D:\\other\\src\\|layer1",
      "just a regular string without any paths",
      "D:\\other\\src\\文本.txt",
      "ignored|.\\a\\b\\c\\d?query=.\\e\\f\\g",
      "D:\\other\\src\\loader.js!D:\\other\\index.js",
      "C:\\Users".into(),
      "C:\\".into(),
      "C:\\Windows".into(),
      r"D:/other/path with space/a.js".into(),
      r"D:\\other\\src\\a.js".into(),
      "D:\\other\\a.js|\0root\\\0|\0escape".into(),
      "D:\\\\other\\\\a.js|\0root\\\\\0".into(),
    ]
  );
}

#[test]
fn test_literal_markers() {
  let root = std::env::temp_dir().join("rspack-marker-project");
  let other_root = std::env::temp_dir().join("rspack-marker-relocated");
  let context = TestContext(Some(root.clone()));
  let other_context = TestContext(Some(other_root.clone()));
  let no_root = TestContext(None);
  let literals = [
    "",
    "文本",
    "\0",
    "\0root/\0",
    "\0root\\\0",
    "\0root\\\\\0",
    "\0escape",
    "\0escape\0escapeescape\0",
    "\0root/\0\0root/\0",
    "\0root/\0|\0escape|\0root\\\0|\0root\\\\\0",
    "\0\0root/\0escape\0root/\0\0文本",
  ];
  let data = Data {
    paths: literals
      .iter()
      .flat_map(|left| literals.iter().map(move |right| format!("{left}{right}")))
      .collect(),
  };
  for encode_context in [&no_root, &context] {
    let bytes = rspack_cacheable::to_bytes(&data, encode_context).unwrap();
    for decode_context in [&no_root, &context, &other_context] {
      let decoded: Data = rspack_cacheable::from_bytes(&bytes, decode_context).unwrap();
      assert_eq!(decoded, data);
    }
  }

  // NormalModule identifiers append the layer verbatim after the resource.
  for literal in literals {
    let path = root.join("src/a.js").to_string_lossy().into_owned();
    let relocated = other_root.join("src/a.js").to_string_lossy().into_owned();
    let outside = root.with_file_name("outside").join("b.js");
    let outside = outside.to_string_lossy();
    let data = Data {
      paths: vec![
        format!("javascript/auto|{path}|{literal}"),
        format!("{literal}|{path}!{path}|{literal}"),
        format!("{outside}|{literal}|{path}"),
      ],
    };
    let (bytes, bound) = rspack_cacheable::to_bytes_with_binding(&data, &context).unwrap();
    assert!(bound);
    let decoded: Data = rspack_cacheable::from_bytes(&bytes, &context).unwrap();
    assert_eq!(decoded, data);
    let decoded: Data = rspack_cacheable::from_bytes(&bytes, &other_context).unwrap();
    assert_eq!(
      decoded.paths,
      vec![
        format!("javascript/auto|{relocated}|{literal}"),
        format!("{literal}|{relocated}!{relocated}|{literal}"),
        format!("{outside}|{literal}|{relocated}"),
      ]
    );
    let bytes = rspack_cacheable::to_bytes(&data, &no_root).unwrap();
    for decode_context in [&no_root, &context, &other_context] {
      let decoded: Data = rspack_cacheable::from_bytes(&bytes, decode_context).unwrap();
      assert_eq!(decoded, data);
    }
  }
}

fn path_string(path: PathBuf) -> String {
  path.to_string_lossy().into_owned()
}

#[test]
fn test_root_with_space() {
  let temp = std::env::temp_dir();
  let root = temp.join("my project");
  let relocated = temp.join("other place").join("my project");
  let identifiers = |root: &Path| Data {
    paths: vec![
      format!(
        "javascript/auto|{}",
        path_string(root.join("src").join("a.js"))
      ),
      path_string(root.join("dir with space").join("b.js")),
      format!("a-loader!{}?q", path_string(root.join("x.js"))),
      format!(
        "{}!{}?from={}",
        path_string(root.join("loader.js")),
        path_string(root.join("x.js")),
        path_string(root.join("y z.js"))
      ),
    ],
  };

  let data = identifiers(&root);
  let (bytes, bound) =
    rspack_cacheable::to_bytes_with_binding(&data, &TestContext(Some(root.clone()))).unwrap();
  assert!(!bound);
  let decoded: Data =
    rspack_cacheable::from_bytes(&bytes, &TestContext(Some(relocated.clone()))).unwrap();
  assert_eq!(decoded, identifiers(&relocated));

  let outside = Data {
    paths: vec![format!(
      "javascript/auto|{}",
      path_string(temp.join("outside dir").join("a.js"))
    )],
  };
  let (bytes, bound) =
    rspack_cacheable::to_bytes_with_binding(&outside, &TestContext(Some(root))).unwrap();
  assert!(bound);
  let decoded: Data = rspack_cacheable::from_bytes(&bytes, &TestContext(Some(relocated))).unwrap();
  assert_eq!(decoded, outside);
}
