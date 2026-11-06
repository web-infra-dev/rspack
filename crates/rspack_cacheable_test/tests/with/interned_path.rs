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

/// Resolver dependency lists may store paths above the root relative to it; plain paths may not.
#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Factorized {
  resolved: InternedPathList,
  registered: Vec<InternedPath>,
}

fn relocate_above_root(paths: &[&str], from: &str, to: &str) -> Vec<InternedPath> {
  let module = Module {
    paths: paths.iter().copied().map(InternedPath::from).collect(),
  };
  let bytes = rspack_cacheable::to_bytes_above_root(
    &module,
    &Context {
      project_root: PathBuf::from(from),
    },
  )
  .expect("should serialize");
  let restored: Module = from_bytes(
    &bytes,
    &Context {
      project_root: PathBuf::from(to),
    },
  )
  .expect("should deserialize");
  restored.paths
}

#[test]
fn test_interned_path_above_root_relocates_by_layout_when_allowed() {
  // Resolver lookups walk up past the root, so a worktree's `../node_modules` miss restores
  // against the new root's parent, and a lookup at the filesystem root restores as itself.
  assert_eq!(
    relocate_above_root(
      &[
        "/home/user/node_modules",
        "/home/user/node_modules/pkg/index.js",
        "/home/shared/lib.js",
        "/home/user/project-two/x.js",
        "/x.js",
        "/home/user/project",
      ],
      ROOT_A,
      ROOT_B,
    ),
    [
      "/w/x/y/z/node_modules",
      "/w/x/y/z/node_modules/pkg/index.js",
      "/w/x/y/shared/lib.js",
      "/w/x/y/z/project-two/x.js",
      "/x.js",
      "/w/x/y/z/proj",
    ]
    .map(InternedPath::from)
  );
}

#[test]
fn test_interned_path_above_root_restores_at_a_shallower_root() {
  // The writer at ROOT_B recorded a miss three levels up. A reader whose root has fewer ancestors
  // must still decode; the path clamps at the filesystem root, where the snapshot finds it absent.
  assert_eq!(
    relocate_above_root(&["/w/x/node_modules"], ROOT_B, "/ci/proj"),
    vec![InternedPath::from("/node_modules")]
  );
}

#[test]
fn test_resolver_lists_rebase_above_root_but_registered_paths_bind() {
  let context = Context {
    project_root: PathBuf::from(ROOT_A),
  };
  let above = InternedPath::from("/home/user/node_modules");
  let resolved_only = Factorized {
    resolved: InternedPathList::from_vec(vec![above.clone()]),
    registered: vec![],
  };
  let (bytes, bound) = to_bytes_with_binding(&resolved_only, &context).expect("should serialize");
  assert!(!bound, "a resolver lookup above the root must not bind");
  let restored: Factorized = from_bytes(
    &bytes,
    &Context {
      project_root: PathBuf::from(ROOT_B),
    },
  )
  .expect("should deserialize");
  assert_eq!(
    restored.resolved.as_slice().to_vec(),
    vec![InternedPath::from("/w/x/y/z/node_modules")]
  );

  // A loader may register a fixed absolute path; its contents are not a function of the root.
  let registered = Factorized {
    resolved: InternedPathList::from_vec(vec![]),
    registered: vec![above],
  };
  let (_, bound) = to_bytes_with_binding(&registered, &context).expect("should serialize");
  assert!(
    bound,
    "a registered path above the root keeps binding its entry"
  );
}

#[test]
fn test_shared_resolver_path_still_binds_when_registered_by_a_loader() {
  let above = InternedPath::from("/home/user/node_modules");
  let module = Factorized {
    resolved: InternedPathList::from_vec(vec![above.clone()]),
    registered: vec![above],
  };
  let (_, bound) = to_bytes_with_binding(
    &module,
    &Context {
      project_root: PathBuf::from(ROOT_A),
    },
  )
  .expect("should serialize");
  assert!(
    bound,
    "sharing must not bypass a registered path's root binding"
  );
}

#[cfg(unix)]
#[test]
fn test_above_root_restores_at_the_filesystem_root() {
  assert_eq!(
    relocate_above_root(&["/package.json", "/home/package.json"], ROOT_A, "/"),
    ["/package.json", "/package.json"].map(InternedPath::from)
  );
}

#[cfg(windows)]
#[test]
fn test_above_root_preserves_windows_filesystem_prefixes() {
  for (writer, reader, lookup, expected) in [
    (
      r"C:\a\proj",
      r"C:\b\proj",
      r"C:\package.json",
      r"C:\package.json",
    ),
    (
      r"\\server\share\a\proj",
      r"\\server\share\b\proj",
      r"\\server\share\package.json",
      r"\\server\share\package.json",
    ),
    (
      r"\\?\UNC\server\share\a\proj",
      r"\\?\UNC\server\share\b\proj",
      r"\\?\UNC\server\share\package.json",
      r"\\?\UNC\server\share\package.json",
    ),
  ] {
    assert_eq!(
      relocate_above_root(&[lookup], writer, reader),
      vec![InternedPath::from(expected)]
    );
  }
}

#[cfg(unix)]
#[test]
fn test_fixed_paths_bind_and_keep_their_original_location_inside_and_outside_root() {
  let writer = Context {
    project_root: ROOT_A.into(),
  };
  let reader = Context {
    project_root: ROOT_B.into(),
  };
  for value in ["/home/user/project/src/fixed.js", "/home/user/optional.js"] {
    let path = InternedPath::from(value);
    for fixed_first in [false, true] {
      // Equal filesystem identities retain independent authority even in the same archive.
      let mut paths = vec![path.clone(), path.clone().into_fixed()];
      if fixed_first {
        paths.reverse();
      }
      let module = Module { paths };
      let (bytes, bound) = to_bytes_with_binding(&module, &writer).unwrap();
      assert!(bound);
      let restored: Module = from_bytes(&bytes, &reader).unwrap();
      let fixed = &restored.paths[usize::from(!fixed_first)];
      assert!(fixed.is_fixed());
      assert_eq!(fixed.as_path(), Path::new(value));
      assert!(!restored.paths[usize::from(fixed_first)].is_fixed());
    }
  }
}

#[cfg(unix)]
#[test]
fn test_list_interning_does_not_share_fixed_observations_with_issuer_observations() {
  let path = InternedPath::from("/home/user/project/src/a.js");
  let issuer = InternedPathList::from_vec(vec![path.clone()]);
  let fixed = InternedPathList::from_vec(vec![path.into_fixed()]);
  assert_ne!(issuer, fixed);
  let writer = Context {
    project_root: ROOT_A.into(),
  };
  let reader = Context {
    project_root: ROOT_B.into(),
  };
  for (list, expected, bound) in [
    (&issuer, "/w/x/y/z/proj/src/a.js", false),
    (&fixed, "/home/user/project/src/a.js", true),
  ] {
    let (bytes, actual_bound) = to_bytes_with_binding(list, &writer).unwrap();
    assert_eq!(actual_bound, bound);
    let restored: InternedPathList = from_bytes(&bytes, &reader).unwrap();
    assert_eq!(restored[0].as_path(), Path::new(expected));
    assert_eq!(restored[0].is_fixed(), bound);
  }
}

#[cfg(unix)]
#[test]
fn test_sharing_does_not_grant_above_root_permission_to_plain_paths_in_either_order() {
  #[cacheable]
  struct Observations {
    before: Vec<InternedPath>,
    issuer: InternedPathList,
    after: Vec<InternedPath>,
  }
  let above = InternedPath::from("/home/user/node_modules");
  let observations = Observations {
    before: vec![above.clone()],
    issuer: InternedPathList::from_vec(vec![above.clone()]),
    after: vec![above],
  };
  let (bytes, bound) = to_bytes_with_binding(
    &observations,
    &Context {
      project_root: ROOT_A.into(),
    },
  )
  .unwrap();
  assert!(bound);
  let restored: Observations = from_bytes(
    &bytes,
    &Context {
      project_root: ROOT_B.into(),
    },
  )
  .unwrap();
  assert_eq!(
    restored.before[0].as_path(),
    Path::new("/home/user/node_modules")
  );
  assert_eq!(restored.after[0], restored.before[0]);
  assert_eq!(
    restored.issuer[0].as_path(),
    Path::new("/w/x/y/z/node_modules")
  );
}
