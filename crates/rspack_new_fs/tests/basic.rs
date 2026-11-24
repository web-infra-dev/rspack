use std::{collections::HashMap, hash::BuildHasherDefault, mem::size_of, path::Path, sync::Arc};

use rspack_cacheable::{cacheable, from_bytes, to_bytes};
use rspack_new_fs::{
  AbsPath, AbsPathMap, DiskFileSystem, Error, FileMetadata, IdentityHasher, ProjectPath,
  ProjectPathParts, ProjectPathSet, ReadableFileSystem, Result, RootTable,
};

fn same_project_path_alloc(a: &ProjectPath, b: &ProjectPath) -> bool {
  match (a.parts(), b.parts()) {
    (ProjectPathParts::Anchored { root: r1, .. }, ProjectPathParts::Anchored { root: r2, .. }) => {
      std::ptr::eq(r1.as_ptr(), r2.as_ptr())
    }
    (ProjectPathParts::External(s1), ProjectPathParts::External(s2)) => {
      std::ptr::eq(s1.as_ptr(), s2.as_ptr())
    }
    _ => false,
  }
}

#[derive(Debug, Default)]
struct MockFs {
  files: HashMap<AbsPath, Vec<u8>, BuildHasherDefault<IdentityHasher>>,
  symlinks: HashMap<AbsPath, AbsPath, BuildHasherDefault<IdentityHasher>>,
}

impl MockFs {
  fn resolve(&self, path: &AbsPath) -> AbsPath {
    for (link, target) in &self.symlinks {
      if let Some(rel) = path.strip_prefix(link) {
        return if rel.is_empty() {
          target.clone()
        } else {
          target.join(rel).expect("valid rel")
        };
      }
    }
    path.clone()
  }
}

#[async_trait::async_trait]
impl ReadableFileSystem for MockFs {
  async fn read(&self, path: &AbsPath) -> Result<Vec<u8>> {
    self.read_sync(path)
  }

  fn read_sync(&self, path: &AbsPath) -> Result<Vec<u8>> {
    let resolved = self.resolve(path);
    self.files.get(&resolved).cloned().ok_or_else(|| {
      Error::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "not found",
      ))
    })
  }

  async fn metadata(&self, path: &AbsPath) -> Result<FileMetadata> {
    self.metadata_sync(path)
  }

  fn metadata_sync(&self, path: &AbsPath) -> Result<FileMetadata> {
    let resolved = self.resolve(path);
    if self.files.contains_key(&resolved) {
      return Ok(FileMetadata {
        is_file: true,
        is_directory: false,
        is_symlink: false,
        atime_ms: 0,
        mtime_ms: 0,
        ctime_ms: 0,
        size: self.files[&resolved].len() as u64,
      });
    }
    Err(Error::Io(std::io::Error::new(
      std::io::ErrorKind::NotFound,
      "not found",
    )))
  }

  async fn symlink_metadata(&self, path: &AbsPath) -> Result<FileMetadata> {
    self.metadata_sync(path)
  }

  async fn read_link(&self, path: &AbsPath) -> Result<String> {
    self
      .symlinks
      .get(path)
      .map(|t| t.as_str().to_string())
      .ok_or_else(|| {
        Error::Io(std::io::Error::new(
          std::io::ErrorKind::NotFound,
          "not a symlink",
        ))
      })
  }

  async fn canonicalize(&self, path: &AbsPath) -> Result<AbsPath> {
    Ok(self.canonicalize_sync(path)?)
  }

  fn canonicalize_sync(&self, path: &AbsPath) -> Result<AbsPath> {
    Ok(self.resolve(path))
  }

  async fn read_dir(&self, dir: &AbsPath) -> Result<Vec<String>> {
    self.read_dir_sync(dir)
  }

  fn read_dir_sync(&self, dir: &AbsPath) -> Result<Vec<String>> {
    let resolved = self.resolve(dir);
    Ok(
      self
        .files
        .keys()
        .filter_map(|p| {
          p.strip_prefix(&resolved)
            .filter(|rel| !rel.is_empty())
            .and_then(|rel| rel.split(['/', '\\']).next())
            .map(str::to_owned)
        })
        .collect(),
    )
  }
}

#[test]
fn abs_path_basics() {
  assert_eq!(size_of::<AbsPath>(), size_of::<*const ()>());
  assert_eq!(size_of::<Option<AbsPath>>(), size_of::<*const ()>());

  assert!(matches!(AbsPath::new(""), Err(Error::NotAbsolutePath(_))));
  assert!(matches!(
    AbsPath::new("src/index.ts"),
    Err(Error::NotAbsolutePath(_))
  ));

  #[cfg(unix)]
  {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
    let bad = Path::new(OsStr::from_bytes(b"/tmp/\xff/a.js"));
    assert!(matches!(
      AbsPath::try_from_std_path(bad),
      Err(Error::NonUtf8Path(_))
    ));
  }

  let a = AbsPath::new(if cfg!(windows) {
    r"\\?\c:\Repo\src\..\src\index.ts"
  } else {
    "/repo//src/../src/./index.ts"
  })
  .expect("a");
  let b = AbsPath::new(if cfg!(windows) {
    r"C:/Repo/src/index.ts"
  } else {
    "/repo/src/index.ts"
  })
  .expect("b");
  assert_eq!(a, b);
  assert!(std::ptr::eq(a.as_str().as_ptr(), b.as_str().as_ptr()));

  let repo = a.parent().expect("src").parent().expect("repo");
  assert_eq!(
    a.strip_prefix(&repo),
    Some(if cfg!(windows) {
      r"src\index.ts"
    } else {
      "src/index.ts"
    })
  );
  assert_eq!(repo.join("src/index.ts").expect("join"), a);
  assert_eq!(
    repo.join("a/../../outside/x.js").expect("clamp").as_str(),
    if cfg!(windows) {
      r"C:\outside\x.js"
    } else {
      "/outside/x.js"
    }
  );

  let mut map = AbsPathMap::default();
  map.insert(a, 1);
  assert_eq!(map.get(&b), Some(&1));
}

#[test]
fn project_path_basics() {
  assert_eq!(size_of::<ProjectPath>(), size_of::<*const ()>());
  assert_eq!(size_of::<Option<ProjectPath>>(), size_of::<*const ()>());

  assert!(ProjectPath::anchored("bad/root", "a.ts").is_err());
  assert!(ProjectPath::anchored("project", "../escape").is_err());

  let root = ProjectPath::anchored("project", "").expect("root");
  assert_eq!(
    root.parts(),
    ProjectPathParts::Anchored {
      root: "project",
      rel: ""
    }
  );
  assert!(root.is_anchored());
  assert!(!root.is_external());
  assert_eq!(root.parent_in_root(), None);

  let idx = root
    .try_join("src")
    .expect("src")
    .try_join("./a/../index.ts")
    .expect("idx");
  let expected = ProjectPath::anchored("project", "src/index.ts").expect("expected");
  assert_eq!(idx, expected);
  assert!(same_project_path_alloc(&idx, &expected));
  assert_eq!(idx.try_join("../../../outside"), None);
  if cfg!(unix) {
    assert_eq!(idx.try_join(r"./a\b"), None);
  }

  let back_to_root = idx
    .parent_in_root()
    .expect("src")
    .parent_in_root()
    .expect("root");
  assert!(same_project_path_alloc(&back_to_root, &root));

  let ext = ProjectPath::external(
    &AbsPath::new(if cfg!(windows) {
      r"D:\ext\helper.js"
    } else {
      "/ext/helper.js"
    })
    .expect("ext"),
  );
  assert!(ext.is_external());
  assert_eq!(ext.try_join("sub"), None);
  assert_eq!(ext.parent_in_root(), None);

  let mut set = ProjectPathSet::default();
  set.insert(idx);
  assert!(set.contains(&expected));
}

#[test]
fn project_path_cacheable_roundtrip_and_corruption() {
  #[cacheable]
  #[derive(Debug, PartialEq, Eq)]
  struct Pair(ProjectPath, ProjectPath, ProjectPath);

  let anchored = ProjectPath::anchored("project", "src/shared.ts").expect("anchored");
  let root_only = ProjectPath::anchored("gvs", "").expect("root_only");
  let ext = ProjectPath::external(
    &AbsPath::new(if cfg!(windows) {
      r"D:\ext\lib.js"
    } else {
      "/ext/lib.js"
    })
    .expect("ext"),
  );

  let pair = Pair(anchored.clone(), anchored.clone(), root_only.clone());
  let bytes = to_bytes(&pair, &()).expect("serialize");
  let restored: Pair = from_bytes(&bytes, &()).expect("deserialize");
  assert_eq!(restored, Pair(anchored, restored.0.clone(), root_only));
  assert!(same_project_path_alloc(&restored.0, &restored.1));

  // Verify External roundtrip & wire corruption rejection.
  let ext_bytes = to_bytes(&ext, &()).expect("serialize ext");
  assert_eq!(
    from_bytes::<ProjectPath, _>(&ext_bytes, &()).expect("de ext"),
    ext
  );

  let marker = b"project/src/shared.ts";
  let valid = to_bytes(&restored.0, &()).expect("serialize anchored");
  let pos = valid
    .windows(marker.len())
    .position(|w| w == marker)
    .expect("marker");

  // Bad tag claiming External with relative payload.
  let mut bad_tag = valid.clone();
  bad_tag[pos - 1] = 0;
  assert!(from_bytes::<ProjectPath, _>(&bad_tag, &()).is_err());

  // Invalid UTF-8 payload.
  let mut bad_utf8 = valid.clone();
  bad_utf8[pos + 8] = 0xFF;
  assert!(from_bytes::<ProjectPath, _>(&bad_utf8, &()).is_err());

  // Disallowed `..` inside Anchored rel.
  let mut bad_rel = valid;
  bad_rel[pos + 8..pos + 12].copy_from_slice(b"../x");
  assert!(from_bytes::<ProjectPath, _>(&bad_rel, &()).is_err());
}

#[tokio::test]
async fn root_table_and_disk_fs_basics() {
  let logical_root = AbsPath::new(if cfg!(windows) {
    r"C:\link_repo"
  } else {
    "/link_repo"
  })
  .expect("logical");
  let canon_root = AbsPath::new(if cfg!(windows) {
    r"C:\real_repo"
  } else {
    "/real_repo"
  })
  .expect("canon");
  let nested = canon_root.join("packages/a").expect("nested");

  let mut mock = MockFs::default();
  mock
    .symlinks
    .insert(logical_root.clone(), canon_root.clone());
  mock.files.insert(
    canon_root.join("src/index.ts").expect("idx"),
    b"export {};".to_vec(),
  );
  let mock = Arc::new(mock);

  // Overlapping canonical sub-root must be rejected.
  assert!(matches!(
    RootTable::from_fs(
      mock.as_ref(),
      vec![
        ("project".into(), logical_root.clone()),
        ("sub".into(), nested),
      ]
    ),
    Err(Error::OverlappingRoot { .. })
  ));

  let dfs = DiskFileSystem::new(mock, vec![("project".into(), logical_root.clone())]).expect("dfs");
  let pp = ProjectPath::anchored("project", "src/index.ts").expect("pp");

  assert_eq!(
    dfs.to_project_path(&logical_root.join("src/index.ts").expect("l")),
    pp
  );
  assert_eq!(
    dfs.to_project_path(&canon_root.join("src/index.ts").expect("c")),
    pp
  );
  assert_eq!(
    dfs
      .to_project_path_str(canon_root.join("src/index.ts").expect("c2").as_str())
      .expect("str"),
    pp
  );
  assert_eq!(
    dfs.to_abs_path(&pp).expect("abs"),
    logical_root.join("src/index.ts").expect("exp")
  );

  assert_eq!(dfs.read(&pp).await.expect("read"), b"export {};");
  assert!(dfs.metadata(&pp).await.expect("meta").is_file);
  assert_eq!(dfs.canonicalize(&pp).await.expect("canon"), pp);
  assert_eq!(
    dfs
      .read_dir(&ProjectPath::anchored("project", "src").expect("src"))
      .await
      .expect("read_dir"),
    vec!["index.ts"]
  );

  // Parent above root falls back to External, and join across root boundary preserves External.
  let proj_root = ProjectPath::anchored("project", "").expect("proj");
  assert_eq!(
    dfs.parent(&proj_root).expect("parent"),
    Some(ProjectPath::external(
      &logical_root.parent().expect("parent abs")
    ))
  );
  assert!(
    dfs
      .join(&proj_root, "../outside/x.ts")
      .expect("join out")
      .is_external()
  );

  // Cross-OS External deserialized path fails `to_abs_path`.
  let host_marker = if cfg!(windows) {
    r"D:\cross_os_ext\a\b.ts"
  } else {
    "/cross_os_ext_probe.js"
  };
  let foreign_marker = if cfg!(windows) {
    "/cross_os_ext_probe.js"
  } else {
    r"D:\cross_os_ext\a\b.ts"
  };
  let mut wire = to_bytes(
    &ProjectPath::external(&AbsPath::new(host_marker).expect("h")),
    &(),
  )
  .expect("ser");
  let pos = wire
    .windows(host_marker.len())
    .position(|w| w == host_marker.as_bytes())
    .expect("marker");
  wire[pos..pos + foreign_marker.len()].copy_from_slice(foreign_marker.as_bytes());
  let foreign: ProjectPath = from_bytes(&wire, &()).expect("de");
  assert!(matches!(
    dfs.to_abs_path(&foreign),
    Err(Error::NotAbsolutePath(_))
  ));
}
