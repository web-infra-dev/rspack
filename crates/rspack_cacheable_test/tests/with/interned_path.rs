use std::path::{Path, PathBuf};

use rspack_cacheable::{
  CacheableContext, Error, enable_cacheable as cacheable, from_bytes, to_bytes,
  to_bytes_with_binding,
};
use rspack_paths::{InternedPath, InternedPathList, strip_root_prefix};

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Module {
  paths: Vec<InternedPath>,
}

struct Context {
  project_root: PathBuf,
}

impl CacheableContext for Context {
  fn project_root(&self) -> Option<&Path> {
    Some(&self.project_root)
  }
}

const ROOT_A: &str = "/home/user/project";
const ROOT_B: &str = "/w/x/y/z/proj";

fn relocate(paths: &[&str]) -> Vec<InternedPath> {
  let module = Module {
    paths: paths.iter().copied().map(InternedPath::from).collect(),
  };
  let bytes = to_bytes(
    &module,
    &Context {
      project_root: PathBuf::from(ROOT_A),
    },
  )
  .expect("should serialize interned paths relative to the source root");
  let restored: Module = from_bytes(
    &bytes,
    &Context {
      project_root: PathBuf::from(ROOT_B),
    },
  )
  .expect("should deserialize interned paths relative to the relocated root");
  restored.paths
}

#[test]
fn test_interned_path_under_root_relocates_to_another_depth() {
  assert_eq!(
    relocate(&[
      "/home/user/project/src/a.js",
      "/home/user/project",
      "/home/user/project/node_modules/.pnpm/x/index.js",
    ]),
    [
      "/w/x/y/z/proj/src/a.js",
      "/w/x/y/z/proj",
      "/w/x/y/z/proj/node_modules/.pnpm/x/index.js",
    ]
    .map(InternedPath::from)
  );
}

#[test]
fn test_interned_path_outside_root_round_trips_verbatim() {
  let outside = [
    "/opt/lib/x.js",
    "/home/user/project-two/x.js",
    "/home/user/project/../other/x.js",
    "relative/x.js",
    "",
  ];
  assert_eq!(relocate(&outside), outside.map(InternedPath::from));
}

#[test]
fn test_interned_path_without_root_round_trips() {
  let module = Module {
    paths: vec![InternedPath::from("/home/user/project/src/a.js")],
  };
  let bytes = to_bytes(&module, &()).unwrap();
  let restored: Module = from_bytes(&bytes, &()).unwrap();
  assert_eq!(restored, module);
}

#[test]
fn test_interned_path_rebased_without_root_is_an_error() {
  let module = Module {
    paths: vec![InternedPath::from("/home/user/project/src/a.js")],
  };
  let bytes = to_bytes(
    &module,
    &Context {
      project_root: PathBuf::from(ROOT_A),
    },
  )
  .unwrap();
  let result = from_bytes::<Module, ()>(&bytes, &());
  assert!(matches!(result, Err(Error::MessageError(_))), "{result:?}");
}

#[cfg(unix)]
#[test]
fn test_interned_path_non_utf8_round_trips() {
  use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

  let in_root = InternedPath::new(Path::new(OsStr::from_bytes(
    b"/home/user/project/src/\xff\xfe.js",
  )));
  let outside = InternedPath::new(Path::new(OsStr::from_bytes(b"/opt/\xff.js")));
  let module = Module {
    paths: vec![in_root, outside.clone()],
  };
  let bytes = to_bytes(
    &module,
    &Context {
      project_root: PathBuf::from(ROOT_A),
    },
  )
  .unwrap();
  let restored: Module = from_bytes(
    &bytes,
    &Context {
      project_root: PathBuf::from(ROOT_B),
    },
  )
  .unwrap();
  assert_eq!(
    restored.paths,
    [
      InternedPath::new(Path::new(OsStr::from_bytes(
        b"/w/x/y/z/proj/src/\xff\xfe.js"
      ))),
      outside,
    ]
  );
}

#[test]
fn test_equal_interned_paths_share_one_payload() {
  let context = Context {
    project_root: PathBuf::from(ROOT_A),
  };
  let size = |refs: usize| {
    let module = Module {
      paths: vec![InternedPath::from("/home/user/project/src/a.js"); refs],
    };
    to_bytes(&module, &context).unwrap().len()
  };
  let (one, many) = (size(1), size(1024));
  assert!(
    many - one < 8 * 1023,
    "{} bytes per extra reference",
    (many - one) as f64 / 1023.0
  );
}

#[test]
fn test_interned_path_list_relocates() {
  let list: InternedPathList = ["/home/user/project/src/a.js", "/opt/lib/x.js"]
    .into_iter()
    .map(InternedPath::from)
    .collect();
  let bytes = to_bytes(
    &list,
    &Context {
      project_root: PathBuf::from(ROOT_A),
    },
  )
  .unwrap();
  let restored: InternedPathList = from_bytes(
    &bytes,
    &Context {
      project_root: PathBuf::from(ROOT_B),
    },
  )
  .unwrap();
  assert_eq!(
    restored.as_slice(),
    ["/w/x/y/z/proj/src/a.js", "/opt/lib/x.js"].map(InternedPath::from)
  );
}

#[test]
fn test_interned_path_root_with_trailing_separator_relocates() {
  let module = Module {
    paths: vec![
      InternedPath::from("/home/user/project/src/a.js"),
      InternedPath::from("/home/user/project"),
    ],
  };
  let bytes = to_bytes(
    &module,
    &Context {
      project_root: PathBuf::from("/home/user/project/"),
    },
  )
  .unwrap();
  let restored: Module = from_bytes(
    &bytes,
    &Context {
      project_root: PathBuf::from(ROOT_B),
    },
  )
  .unwrap();
  assert_eq!(
    restored.paths,
    ["/w/x/y/z/proj/src/a.js", "/w/x/y/z/proj"].map(InternedPath::from)
  );
}

#[cfg(unix)]
#[test]
fn test_interned_path_filesystem_root_relocates() {
  let module = Module {
    paths: ["/opt/lib/x.js", "/"].map(InternedPath::from).to_vec(),
  };
  let bytes = to_bytes(
    &module,
    &Context {
      project_root: PathBuf::from("/"),
    },
  )
  .unwrap();
  let restored: Module = from_bytes(
    &bytes,
    &Context {
      project_root: PathBuf::from(ROOT_B),
    },
  )
  .unwrap();
  assert_eq!(
    restored.paths,
    ["/w/x/y/z/proj/opt/lib/x.js", ROOT_B].map(InternedPath::from)
  );
  let bytes = to_bytes(
    &restored,
    &Context {
      project_root: PathBuf::from(ROOT_B),
    },
  )
  .unwrap();
  let restored: Module = from_bytes(
    &bytes,
    &Context {
      project_root: PathBuf::from("/"),
    },
  )
  .unwrap();
  assert_eq!(restored, module);
}

#[cfg(unix)]
#[test]
fn test_interned_path_binding_signal() {
  let context = Context {
    project_root: PathBuf::from(ROOT_A),
  };
  for (path, bound) in [
    ("/opt/lib/x.js", true),
    ("/home/user/project-two/x.js", true),
    ("/home/user/project/../other/x.js", true),
    ("/home/user/project/src/../../other.js", true),
    ("/home/user/project/src/a.js", false),
    ("/home/user/project/src/..name.js", false),
    ("relative/x.js", false),
    ("", false),
  ] {
    let module = Module {
      paths: vec![InternedPath::from(path); 3],
    };
    assert_eq!(
      to_bytes_with_binding(&module, &context).unwrap().1,
      bound,
      "{path}"
    );
  }
}

#[cfg(windows)]
#[test]
fn test_interned_path_windows_native_separators() {
  let module = Module {
    paths: [
      r"C:\repo\src\file.js",
      r"C:\repo\src/part\file.js",
      r"C:\repo-other\file.js",
      r"C:\repo\..\file.js",
    ]
    .map(InternedPath::from)
    .to_vec(),
  };
  let (bytes, bound) = to_bytes_with_binding(
    &module,
    &Context {
      project_root: PathBuf::from(r"C:\repo"),
    },
  )
  .unwrap();
  assert!(bound);
  let restored: Module = from_bytes(
    &bytes,
    &Context {
      project_root: PathBuf::from(r"D:\deeper\repo"),
    },
  )
  .unwrap();
  assert_eq!(
    restored.paths,
    [
      r"D:\deeper\repo\src\file.js",
      r"D:\deeper\repo\src/part\file.js",
      r"C:\repo-other\file.js",
      r"C:\repo\..\file.js"
    ]
    .map(InternedPath::from)
  );
}

#[test]
fn test_strip_root_prefix_accepts_either_windows_separator() {
  let root = br"C:\work\project";
  let windows = b"\\/";
  for (path, suffix) in [
    (&br"C:/work/project/src/a.js"[..], Some(&b"/src/a.js"[..])),
    (br"C:\work\project\src\a.js", Some(br"\src\a.js")),
    (br"C:\work/project\src/a.js", Some(br"\src/a.js")),
    (br"C:\\work//project", Some(b"")),
    (br"C:/work/project-two/a.js", None),
    (br"C:/work/project/../a.js", None),
    (br"D:/work/project/src/a.js", None),
  ] {
    assert_eq!(
      strip_root_prefix(path, root, windows),
      suffix,
      "{}",
      String::from_utf8_lossy(path)
    );
  }
  assert_eq!(
    strip_root_prefix(b"C:/work/project/src/a.js", root, b"/"),
    None
  );
}

#[cfg(windows)]
#[test]
fn test_interned_path_windows_forward_slash_spelling_relocates() {
  let module = Module {
    paths: vec![InternedPath::from("C:/work/project/src/a.js")],
  };
  let (bytes, bound) = to_bytes_with_binding(
    &module,
    &Context {
      project_root: PathBuf::from(r"C:\work\project"),
    },
  )
  .unwrap();
  assert!(!bound);
  let restored: Module = from_bytes(
    &bytes,
    &Context {
      project_root: PathBuf::from(r"D:\deeper\repo"),
    },
  )
  .unwrap();
  assert_eq!(
    restored.paths,
    [InternedPath::from(r"D:\deeper\repo\src\a.js")]
  );
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

#[cfg(unix)]
#[test]
fn test_interned_path_under_either_root_spelling_relocates() {
  let writer = AliasedContext {
    root: PathBuf::from("/tmp/a/proj"),
    canonical_root: PathBuf::from("/private/tmp/a/proj"),
  };
  let reader = AliasedContext {
    root: PathBuf::from("/tmp/b/x/proj"),
    canonical_root: PathBuf::from("/private/tmp/b/x/proj"),
  };
  let module = Module {
    paths: ["/private/tmp/a/proj/src/a.js", "/tmp/a/proj/src/b.js"]
      .map(InternedPath::from)
      .to_vec(),
  };
  let (bytes, bound) = to_bytes_with_binding(&module, &writer).unwrap();
  assert!(!bound);
  let restored: Module = from_bytes(&bytes, &reader).unwrap();
  assert_eq!(
    restored.paths,
    ["/private/tmp/b/x/proj/src/a.js", "/tmp/b/x/proj/src/b.js"].map(InternedPath::from)
  );
}
