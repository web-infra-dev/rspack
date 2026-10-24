use std::path::{Path, PathBuf};

use cow_utils::CowUtils;
#[cfg(not(windows))]
use rspack_cacheable::ContextIdentifierLayout;
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
    "\0realpath/\0",
    "\0realpath\\\0",
    "\0realpath\\\\\0",
    "\0realpath/\0\0root/\0escape",
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

/// A root reached through an alias such as a symlink: `context` spells it one way and the
/// resolver realpaths resources to the other.
struct AliasedContext {
  root: PathBuf,
  canonical_root: PathBuf,
}

impl AliasedContext {
  fn new(root: PathBuf, canonical_root: PathBuf) -> Self {
    Self {
      root,
      canonical_root,
    }
  }
}

impl CacheableContext for AliasedContext {
  fn project_root(&self) -> Option<&Path> {
    Some(&self.root)
  }

  fn canonical_project_root(&self) -> Option<&Path> {
    Some(&self.canonical_root)
  }
}

fn aliased_identifiers(context: &AliasedContext) -> Data {
  let real = |path: &str| path_string(context.canonical_root.join(path));
  let link = |path: &str| path_string(context.root.join(path));
  Data {
    paths: vec![
      format!("javascript/auto|{}", real("src/a.js")),
      format!("{}!{}", real("loader.js"), link("src/b.js")),
      format!("{}?from={}", link("src/c.js"), real("src/d.js")),
    ],
  }
}

#[test]
fn test_aliased_root_restores_each_spelling() {
  let temp = std::env::temp_dir();
  let writer = AliasedContext::new(
    temp.join("link-a").join("project"),
    temp.join("real-a").join("project"),
  );
  let reader = AliasedContext::new(
    temp.join("link-b").join("deeper").join("project"),
    temp.join("real-b").join("project"),
  );

  let data = aliased_identifiers(&writer);
  let (bytes, bound) = rspack_cacheable::to_bytes_with_binding(&data, &writer).unwrap();
  assert!(!bound);
  let decoded: Data = rspack_cacheable::from_bytes(&bytes, &writer).unwrap();
  assert_eq!(decoded, data);
  let decoded: Data = rspack_cacheable::from_bytes(&bytes, &reader).unwrap();
  assert_eq!(decoded, aliased_identifiers(&reader));

  let unaliased = temp.join("plain").join("project");
  let decoded: Data =
    rspack_cacheable::from_bytes(&bytes, &TestContext(Some(unaliased.clone()))).unwrap();
  assert_eq!(
    decoded,
    aliased_identifiers(&AliasedContext::new(unaliased.clone(), unaliased))
  );

  let outside = Data {
    paths: vec![format!(
      "javascript/auto|{}",
      path_string(temp.join("real-a").join("other").join("a.js"))
    )],
  };
  let (_, bound) = rspack_cacheable::to_bytes_with_binding(&outside, &writer).unwrap();
  assert!(bound);
}

#[test]
fn test_unaliased_root_restores_as_realpath() {
  let temp = std::env::temp_dir();
  let root = temp.join("plain").join("project");
  let data = Data {
    paths: vec![format!(
      "javascript/auto|{}",
      path_string(root.join("src").join("a.js"))
    )],
  };
  let bytes = rspack_cacheable::to_bytes(&data, &TestContext(Some(root))).unwrap();
  let reader = AliasedContext::new(
    temp.join("link-b").join("project"),
    temp.join("real-b").join("project"),
  );
  let decoded: Data = rspack_cacheable::from_bytes(&bytes, &reader).unwrap();
  assert_eq!(
    decoded.paths,
    vec![format!(
      "javascript/auto|{}",
      path_string(reader.canonical_root.join("src").join("a.js"))
    )]
  );
}

#[test]
fn test_nested_spellings_prefer_the_longest() {
  let temp = std::env::temp_dir();
  let writer = AliasedContext::new(temp.join("nest").join("link"), temp.join("nest"));
  let reader = AliasedContext::new(temp.join("elsewhere").join("link"), temp.join("moved"));
  let data = Data {
    paths: vec![
      path_string(writer.root.join("a.js")),
      path_string(writer.canonical_root.join("b.js")),
    ],
  };
  let (bytes, bound) = rspack_cacheable::to_bytes_with_binding(&data, &writer).unwrap();
  assert!(!bound);
  let decoded: Data = rspack_cacheable::from_bytes(&bytes, &reader).unwrap();
  assert_eq!(
    decoded.paths,
    vec![
      path_string(reader.root.join("a.js")),
      path_string(reader.canonical_root.join("b.js")),
    ]
  );
}

#[cfg(not(windows))]
struct StructuredIdentifierContext {
  root: PathBuf,
  identifier: String,
  layout: ContextIdentifierLayout,
}

#[cfg(not(windows))]
fn regex_layout(regex: std::ops::Range<usize>) -> ContextIdentifierLayout {
  ContextIdentifierLayout {
    regexes: vec![regex],
    paths: Vec::new(),
  }
}

#[cfg(not(windows))]
impl CacheableContext for StructuredIdentifierContext {
  fn project_root(&self) -> Option<&Path> {
    Some(&self.root)
  }

  fn context_identifier_layout(&self, value: &str) -> Option<&ContextIdentifierLayout> {
    (value == self.identifier).then_some(&self.layout)
  }
}

#[test]
#[cfg(not(windows))]
fn test_typed_context_regex_does_not_hide_outside_paths() {
  let pattern = r"/^\.\/.*$/";
  for (identifier, bound) in [
    (format!("/project/dir|sync|{pattern}"), false),
    (format!("/outside/dir|sync|{pattern}"), true),
    (
      format!("context|ctx request./dir true {pattern}   sync  "),
      false,
    ),
    (
      format!("context|ctx request./dir true {pattern} /outside/file.js  sync  "),
      true,
    ),
    (
      format!("/project/dir|sync|{pattern}|/outside/file.js"),
      true,
    ),
    (
      format!("/outside/loader.js!/project/dir|sync|{pattern}"),
      true,
    ),
  ] {
    let start = identifier.find(pattern).unwrap();
    let context = StructuredIdentifierContext {
      root: "/project".into(),
      identifier: identifier.clone(),
      layout: regex_layout(start..start + pattern.len()),
    };
    let data = rspack_collections::Identifier::from(identifier.as_str());
    let (bytes, actual_bound) = rspack_cacheable::to_bytes_with_binding(&data, &context).unwrap();
    assert_eq!(actual_bound, bound, "{identifier}");
    let generic = Data {
      paths: vec![identifier.clone()],
    };
    let (_, generic_bound) = rspack_cacheable::to_bytes_with_binding(&generic, &context).unwrap();
    assert!(
      generic_bound,
      "ordinary string fields must remain conservative"
    );
    let moved: rspack_collections::Identifier =
      rspack_cacheable::from_bytes(&bytes, &TestContext(Some("/moved/deeper/project".into())))
        .unwrap();
    assert_eq!(
      moved.as_str(),
      identifier.cow_replace("/project/", "/moved/deeper/project/")
    );
  }

  let ambiguous = rspack_collections::Identifier::from(format!("/project/dir|sync|{pattern}"));
  let (_, bound) =
    rspack_cacheable::to_bytes_with_binding(&ambiguous, &TestContext(Some("/project".into())))
      .unwrap();
  assert!(bound);
}

#[test]
#[cfg(not(windows))]
fn test_invalid_context_regex_span_stays_bound() {
  let identifier = "/project/文本|sync|/outside/file.js";
  for regex in [0..0, 9..10, 0..identifier.len() + 1] {
    let context = StructuredIdentifierContext {
      root: "/project".into(),
      identifier: identifier.into(),
      layout: regex_layout(regex),
    };
    let data = rspack_collections::Identifier::from(identifier);
    let (_, bound) = rspack_cacheable::to_bytes_with_binding(&data, &context).unwrap();
    assert!(bound);
  }
}

#[test]
#[cfg(not(windows))]
fn test_typed_context_regex_content_is_not_rewritten() {
  let pattern = "/^/project/.*$/";
  let prefix = "/project/dir|sync|";
  let identifier = format!("{prefix}{pattern}");
  let context = StructuredIdentifierContext {
    root: "/project".into(),
    identifier: identifier.clone(),
    layout: regex_layout(prefix.len()..identifier.len()),
  };
  let data = rspack_collections::Identifier::from(identifier);
  let bytes = rspack_cacheable::to_bytes(&data, &context).unwrap();
  assert!(
    bytes
      .windows(pattern.len())
      .any(|bytes| bytes == pattern.as_bytes())
  );
  let moved: rspack_collections::Identifier =
    rspack_cacheable::from_bytes(&bytes, &TestContext(Some("/moved/project".into()))).unwrap();
  assert_eq!(moved.as_str(), format!("/moved/project/dir|sync|{pattern}"));
}

#[test]
#[cfg(not(windows))]
fn test_typed_context_regex_literal_markers() {
  for literal in [
    "\0",
    "\0root/\0",
    "\0root\\\0",
    "\0root\\\\\0",
    "\0escape",
    "文本\0escape\0root/\0",
  ] {
    let pattern = format!("/^/project/{literal}.*$/");
    for outside in ["", "/outside/file.js"] {
      let prefix = format!("/project/dir|{literal}|sync|");
      let suffix = format!("|{outside}|/project/tail|{literal}");
      let identifier = format!("{prefix}{pattern}{suffix}");
      let context = StructuredIdentifierContext {
        root: "/project".into(),
        identifier: identifier.clone(),
        layout: regex_layout(prefix.len()..prefix.len() + pattern.len()),
      };
      let data = rspack_collections::Identifier::from(identifier.as_str());
      let (bytes, bound) = rspack_cacheable::to_bytes_with_binding(&data, &context).unwrap();
      assert_eq!(bound, !outside.is_empty());
      let restored: rspack_collections::Identifier =
        rspack_cacheable::from_bytes(&bytes, &context).unwrap();
      assert_eq!(restored, data);
      let moved: rspack_collections::Identifier =
        rspack_cacheable::from_bytes(&bytes, &TestContext(Some("/moved/deeper/project".into())))
          .unwrap();
      assert_eq!(
        moved.as_str(),
        format!(
          "{}{pattern}{}",
          prefix.cow_replace("/project/", "/moved/deeper/project/"),
          suffix.cow_replace("/project/", "/moved/deeper/project/")
        )
      );

      let unrelated_context = StructuredIdentifierContext {
        identifier: format!("{identifier}|other"),
        ..context
      };
      let (_, bound) = rspack_cacheable::to_bytes_with_binding(&data, &unrelated_context).unwrap();
      assert!(
        bound,
        "metadata for a different identifier must not exempt its regex"
      );
    }
  }
}

#[test]
#[cfg(not(windows))]
fn test_typed_context_paths_relocate() {
  let regex = r"/^\.\/.*$/";
  let element = "context/home/user/project/dir|./a.js";
  let lazy = format!(
    "/home/user/project/dir|lazy|{regex}|dep=context/home/user/project/dir|./a.js|modifier=./a.js"
  );
  let lazy_once = format!(
    "/home/user/project/dir|lazy-once|{regex}|dep=context/home/user/project/dir|./a.js\
     context/home/user/project/dir|./b.js|loc=1:0-10"
  );
  let outside = "context/opt/dir|./a.js";
  let path_after = |identifier: &str, marker: &str| identifier.find(marker).unwrap() + marker.len();
  let regex_in = |identifier: &str| {
    let start = identifier.find(regex).unwrap();
    let range = start..start + regex.len();
    vec![range]
  };
  for (identifier, regexes, paths, bound) in [
    (element.to_string(), vec![], vec![7], false),
    (
      lazy.clone(),
      regex_in(&lazy),
      vec![path_after(&lazy, "|dep=context")],
      false,
    ),
    (
      lazy_once.clone(),
      regex_in(&lazy_once),
      vec![
        path_after(&lazy_once, "|dep=context"),
        path_after(&lazy_once, "jscontext"),
      ],
      false,
    ),
    (
      lazy.clone(),
      vec![],
      vec![path_after(&lazy, "|dep=context")],
      true,
    ),
    (outside.to_string(), vec![], vec![7], true),
  ] {
    let writer = StructuredIdentifierContext {
      root: "/home/user/project".into(),
      identifier: identifier.clone(),
      layout: ContextIdentifierLayout { regexes, paths },
    };
    let data = rspack_collections::Identifier::from(identifier.as_str());
    let (bytes, actual_bound) = rspack_cacheable::to_bytes_with_binding(&data, &writer).unwrap();
    assert_eq!(actual_bound, bound, "{identifier}");
    let moved: rspack_collections::Identifier =
      rspack_cacheable::from_bytes(&bytes, &TestContext(Some("/w/x/proj".into()))).unwrap();
    assert_eq!(
      moved.as_str(),
      identifier.cow_replace("/home/user/project/", "/w/x/proj/"),
      "{identifier}"
    );
  }

  let untyped = rspack_collections::Identifier::from(element);
  let bytes =
    rspack_cacheable::to_bytes(&untyped, &TestContext(Some("/home/user/project".into()))).unwrap();
  let moved: rspack_collections::Identifier =
    rspack_cacheable::from_bytes(&bytes, &TestContext(Some("/w/x/proj".into()))).unwrap();
  assert_eq!(moved, untyped, "only a typed layout rebases a context path");
}

#[test]
#[cfg(not(windows))]
fn test_context_shaped_requests_stay_verbatim() {
  for (writer, reader, request) in [
    ("/tmp/A", "/tmp/B", "context/tmp/A/pkg"),
    ("/home/user/project", "/w/x/proj", "context/lib"),
    ("/lib", "/w/lib", "context/lib/index.js"),
  ] {
    let writer = TestContext(Some(writer.into()));
    let reader = TestContext(Some(reader.into()));
    let data = Data {
      paths: vec![request.into()],
    };
    let (bytes, bound) = rspack_cacheable::to_bytes_with_binding(&data, &writer).unwrap();
    assert!(!bound, "{request}");
    let restored: Data = rspack_cacheable::from_bytes(&bytes, &reader).unwrap();
    assert_eq!(restored, data);

    let identifier = rspack_collections::Identifier::from(request);
    let (bytes, bound) = rspack_cacheable::to_bytes_with_binding(&identifier, &writer).unwrap();
    assert!(!bound, "{request}");
    let restored: rspack_collections::Identifier =
      rspack_cacheable::from_bytes(&bytes, &reader).unwrap();
    assert_eq!(restored, identifier);
  }
}
