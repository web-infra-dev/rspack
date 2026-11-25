use std::{
  collections::HashMap, hash::BuildHasherDefault, io::ErrorKind, mem::size_of, path::Path,
};

use rspack_cacheable::{cacheable, from_bytes, to_bytes};
use rspack_fs::{
  AbsPath, AbsPathMap, Error, FileMetadata, FilePermissions, IdentityHasher,
  IntermediateFileSystem, IntermediateFileSystemExtras, MemoryFileSystem, NativeFileSystem,
  NoopFileSystem, ProjectPath, ProjectPathParts, ProjectPathSet, ReadableFileSystem, Result,
  RootTable, WritableFileSystem,
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

#[test]
fn root_table_basics() {
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

  // Overlapping canonical sub-root must be rejected.
  assert!(matches!(
    RootTable::from_fs(
      &mock,
      vec![
        ("project".into(), logical_root.clone()),
        ("sub".into(), nested),
      ]
    ),
    Err(Error::OverlappingRoot { .. })
  ));

  let table =
    RootTable::from_fs(&mock, vec![("project".into(), logical_root.clone())]).expect("table");
  let pp = ProjectPath::anchored("project", "src/index.ts").expect("pp");

  assert_eq!(
    table.to_project_path(&logical_root.join("src/index.ts").expect("l")),
    pp
  );
  assert_eq!(
    table.to_project_path(&canon_root.join("src/index.ts").expect("c")),
    pp
  );
  assert_eq!(
    table
      .to_project_path_str(canon_root.join("src/index.ts").expect("c2").as_str())
      .expect("str"),
    pp
  );
  assert_eq!(
    table.to_abs_path(&pp).expect("abs"),
    logical_root.join("src/index.ts").expect("exp")
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
    table.to_abs_path(&foreign),
    Err(Error::NotAbsolutePath(_))
  ));
}

#[tokio::test]
async fn memory_fs_read_operations() {
  let fs = MemoryFileSystem::new();
  let test_path = |posix: &str| -> AbsPath {
    if cfg!(windows) {
      let mut win = String::from("C:");
      for seg in posix.trim_start_matches('/').split('/') {
        win.push('\\');
        win.push_str(seg);
      }
      AbsPath::new(&win).expect("win test path")
    } else {
      AbsPath::new(posix).expect("unix test path")
    }
  };

  let dir = test_path("/proj/src");
  let file = test_path("/proj/src/index.ts");
  let sibling = test_path("/proj/src/other.ts");
  let missing = test_path("/proj/src/missing.ts");

  fs.create_dir_all(&dir).expect("create_dir_all");
  assert!(matches!(
    fs.create_dir(&test_path("/proj/src/index.ts/sub")),
    Err(Error::Io(e)) if e.kind() == ErrorKind::NotFound || e.kind() == ErrorKind::NotADirectory
  ));

  fs.write(&file, b"export const x = 1;").expect("write file");
  assert!(matches!(
    fs.write(&dir, b"bad"),
    Err(Error::Io(e)) if e.kind() == ErrorKind::IsADirectory
  ));
  assert!(matches!(
    fs.write(&test_path("/proj/src/index.ts/nested.ts"), b"bad"),
    Err(Error::Io(e)) if e.kind() == ErrorKind::NotADirectory
  ));

  assert_eq!(fs.read(&file).await.expect("read"), b"export const x = 1;");
  assert!(fs.exists(&file));
  assert!(!fs.exists(&missing));
  assert_eq!(
    fs.read_to_string(&file).await.expect("read_to_string"),
    "export const x = 1;"
  );

  let meta1 = fs.metadata(&file).await.expect("meta1");
  assert!(meta1.is_file);
  assert!(!meta1.is_directory);
  assert_eq!(meta1.size, 19);

  std::thread::sleep(std::time::Duration::from_millis(20));
  fs.write(&file, b"export const x = 2;")
    .expect("rewrite file");
  let meta2 = fs.metadata(&file).await.expect("meta2");
  assert!(meta2.mtime_ms > meta1.mtime_ms);

  fs.write(&sibling, b"export const y = 2;")
    .expect("write sibling");
  let mut entries = ReadableFileSystem::read_dir(&fs, &dir)
    .await
    .expect("read_dir");
  entries.sort_unstable();
  assert_eq!(entries, vec!["index.ts", "other.ts"]);

  assert!(matches!(
    ReadableFileSystem::read_dir(&fs, &file).await,
    Err(Error::Io(e)) if e.kind() == ErrorKind::NotADirectory
  ));
  assert!(matches!(
    fs.read(&missing).await,
    Err(Error::Io(e)) if e.kind() == ErrorKind::NotFound
  ));
  assert_eq!(fs.canonicalize(&file).await.expect("canon"), file);
  assert!(matches!(
    fs.canonicalize(&missing).await,
    Err(Error::Io(e)) if e.kind() == ErrorKind::NotFound
  ));
  assert!(matches!(
    fs.read_link(&file).await,
    Err(Error::Io(e)) if e.kind() == ErrorKind::Unsupported
  ));
}

#[tokio::test]
async fn native_fs_read_operations_and_non_utf8_rejection() {
  let temp = tempfile::tempdir().expect("tempdir");
  let root = AbsPath::try_from_std_path(temp.path()).expect("root abs");
  let src = root.join("src").expect("src");
  std::fs::create_dir_all(src.as_std_path()).expect("mkdir");
  let file = src.join("a.txt").expect("file");
  std::fs::write(file.as_std_path(), b"hello native").expect("write");

  let fs = NativeFileSystem::new(false);
  assert_eq!(fs.read(&file).await.expect("read"), b"hello native");
  assert_eq!(fs.read_sync(&file).expect("read_sync"), b"hello native");

  let meta = fs.metadata(&file).await.expect("meta");
  assert!(meta.is_file);
  assert_eq!(meta.size, 12);
  assert_eq!(fs.metadata_sync(&file).expect("meta_sync"), meta);

  let entries = ReadableFileSystem::read_dir(&fs, &src)
    .await
    .expect("read_dir");
  assert_eq!(entries, vec!["a.txt"]);
  assert_eq!(
    fs.read_dir_sync(&src).expect("read_dir_sync"),
    vec!["a.txt"]
  );

  let canon = fs.canonicalize(&file).await.expect("canon");
  assert!(canon.as_str().ends_with("a.txt"));

  let perms = fs.permissions(&file).await.expect("permissions");
  if cfg!(unix) {
    assert!(perms.is_some());
  }

  #[cfg(unix)]
  {
    use std::{
      ffi::OsStr,
      os::unix::{ffi::OsStrExt, fs::symlink},
    };

    let link = src.join("link.txt").expect("link");
    symlink("a.txt", link.as_std_path()).expect("symlink");
    assert!(fs.symlink_metadata(&link).await.expect("lstat").is_symlink);
    let symlink_target = fs.read_link(&link).await.expect("read_link");
    assert_eq!(symlink_target, "a.txt");

    // macOS APFS rejects invalid UTF-8 filenames at the kernel level, so only
    // exercise on-disk non-UTF-8 directory entries where the OS permits creating them.
    let bad_dir = src.join("bad_dir").expect("bad_dir");
    std::fs::create_dir(bad_dir.as_std_path()).expect("mkdir bad");
    let bad_name = OsStr::from_bytes(b"bad_\xff.js");
    if std::fs::write(bad_dir.as_std_path().join(bad_name), b"x").is_ok() {
      assert!(matches!(
        ReadableFileSystem::read_dir(&fs, &bad_dir).await,
        Err(Error::NonUtf8Path(_))
      ));
    }
  }
}

#[test]
fn abs_path_and_root_table_compat_conversions() {
  use std::path::PathBuf;

  use camino::{Utf8Path, Utf8PathBuf};

  let raw_str = if cfg!(windows) {
    r"C:\repo\src\index.ts"
  } else {
    "/repo/src/index.ts"
  };
  let root_str = if cfg!(windows) { r"C:\repo" } else { "/repo" };

  let abs = AbsPath::try_from_utf8_path(Utf8Path::new(raw_str)).expect("utf8 path");
  assert_eq!(abs.as_utf8_path(), Utf8Path::new(raw_str));
  assert_eq!(abs.to_utf8_path_buf(), Utf8PathBuf::from(raw_str));
  assert_eq!(abs.to_std_path_buf(), PathBuf::from(raw_str));
  assert_eq!(abs.file_name(), Some("index.ts"));
  assert_eq!(abs.extension(), Some("ts"));
  assert_eq!(AbsPath::try_from(raw_str).expect("try_from str"), abs);
  assert_eq!(
    AbsPath::try_from(Utf8Path::new(raw_str)).expect("try_from &Utf8Path"),
    abs
  );
  assert_eq!(
    AbsPath::try_from(&Utf8PathBuf::from(raw_str)).expect("try_from &Utf8PathBuf"),
    abs
  );
  assert_eq!(
    AbsPath::try_from(Utf8PathBuf::from(raw_str)).expect("try_from Utf8PathBuf"),
    abs
  );
  assert_eq!(
    AbsPath::try_from(Path::new(raw_str)).expect("try_from &Path"),
    abs
  );
  assert_eq!(
    AbsPath::try_from(&PathBuf::from(raw_str)).expect("try_from &PathBuf"),
    abs
  );
  assert_eq!(
    AbsPath::try_from(PathBuf::from(raw_str)).expect("try_from PathBuf"),
    abs
  );
  assert_eq!(Utf8PathBuf::from(&abs), Utf8PathBuf::from(raw_str));
  assert_eq!(Utf8PathBuf::from(abs.clone()), Utf8PathBuf::from(raw_str));
  assert_eq!(PathBuf::from(&abs), PathBuf::from(raw_str));
  assert_eq!(PathBuf::from(abs), PathBuf::from(raw_str));

  let roots = RootTable::without_canonicalization(vec![(
    "project".to_string(),
    AbsPath::new(root_str).expect("root"),
  )])
  .expect("table");
  let proj = roots
    .to_project_path_utf8(Utf8Path::new(raw_str))
    .expect("project path");
  assert_eq!(
    proj.parts(),
    ProjectPathParts::Anchored {
      root: "project",
      rel: "src/index.ts"
    }
  );
  assert_eq!(
    roots.to_utf8_path_buf(&proj).expect("utf8"),
    Utf8PathBuf::from(raw_str)
  );
  assert_eq!(
    roots.to_std_path_buf(&proj).expect("std"),
    PathBuf::from(raw_str)
  );
}

#[tokio::test]
async fn memory_fs_write_intermediate_and_streams() {
  let fs = MemoryFileSystem::new();
  let root = if cfg!(windows) {
    AbsPath::new(r"C:\work").expect("work")
  } else {
    AbsPath::new("/work").expect("work")
  };
  let sub = root.join("sub").expect("sub");
  let nested = sub.join("nested").expect("nested");
  let file = nested.join("a.txt").expect("file");
  let moved = root.join("moved.txt").expect("moved");

  WritableFileSystem::create_dir(&fs, &root)
    .await
    .expect("create_dir");
  assert!(
    WritableFileSystem::create_dir(&fs, &nested.join("leaf").expect("leaf"))
      .await
      .is_err()
  );
  WritableFileSystem::create_dir_all(&fs, &nested)
    .await
    .expect("create_dir_all");
  WritableFileSystem::write(&fs, &file, b"hello")
    .await
    .expect("write");
  assert_eq!(
    WritableFileSystem::read_file(&fs, &file)
      .await
      .expect("read_file"),
    b"hello"
  );
  assert!(
    WritableFileSystem::stat(&fs, &file)
      .await
      .expect("stat")
      .is_file
  );

  // Removing a missing file succeeds; removing a directory as a file fails.
  WritableFileSystem::remove_file(&fs, &moved)
    .await
    .expect("remove missing ok");
  assert!(matches!(
    WritableFileSystem::remove_file(&fs, &sub).await,
    Err(Error::Io(e)) if e.kind() == ErrorKind::IsADirectory
  ));

  // File rename + same-path + missing source + directory rename + into-self checks.
  IntermediateFileSystemExtras::rename(&fs, &file, &file)
    .await
    .expect("self rename ok");
  IntermediateFileSystemExtras::rename(&fs, &file, &moved)
    .await
    .expect("rename file");
  assert!(matches!(
    WritableFileSystem::stat(&fs, &file).await,
    Err(Error::Io(e)) if e.kind() == ErrorKind::NotFound
  ));
  assert_eq!(
    WritableFileSystem::read_file(&fs, &moved)
      .await
      .expect("read moved"),
    b"hello"
  );
  assert!(matches!(
    IntermediateFileSystemExtras::rename(&fs, &file, &moved).await,
    Err(Error::Io(e)) if e.kind() == ErrorKind::NotFound
  ));

  let moved_dir = root.join("sub2").expect("sub2");
  assert!(matches!(
    IntermediateFileSystemExtras::rename(&fs, &sub, &nested).await,
    Err(Error::Io(e)) if e.kind() == ErrorKind::InvalidInput
  ));
  WritableFileSystem::write(&fs, &nested.join("b.txt").expect("b"), b"b")
    .await
    .expect("write b");
  IntermediateFileSystemExtras::rename(&fs, &sub, &moved_dir)
    .await
    .expect("rename dir");
  assert_eq!(
    WritableFileSystem::read_file(&fs, &moved_dir.join("nested/b.txt").expect("moved b"))
      .await
      .expect("read moved b"),
    b"b"
  );

  // Streams
  let stream_file = root.join("stream.txt").expect("stream");
  let mut writer = IntermediateFileSystemExtras::create_write_stream(&fs, &stream_file)
    .await
    .expect("create write stream");
  writer.write_line("line1").await.expect("write_line");
  writer.write_all(b"line2").await.expect("write_all");
  writer.flush().await.expect("flush");
  writer.close().await.expect("close");

  let mut reader = IntermediateFileSystemExtras::create_read_stream(&fs, &stream_file)
    .await
    .expect("create read stream");
  assert_eq!(reader.read_line().await.expect("read_line"), "line1");
  assert_eq!(reader.read(2).await.expect("read"), b"li");
  reader.skip(1).await.expect("skip");
  assert_eq!(reader.read_to_end().await.expect("read_to_end"), b"e2");
  reader.close().await.expect("close reader");

  // remove_dir_all
  WritableFileSystem::remove_dir_all(&fs, &moved_dir)
    .await
    .expect("remove_dir_all");
  assert!(matches!(
    WritableFileSystem::stat(&fs, &moved_dir).await,
    Err(Error::Io(e)) if e.kind() == ErrorKind::NotFound
  ));

  let as_intermediate: &dyn IntermediateFileSystem = &fs;
  assert!(matches!(
    as_intermediate.stat(&moved_dir).await,
    Err(Error::Io(e)) if e.kind() == ErrorKind::NotFound
  ));
}

#[tokio::test]
async fn native_fs_write_intermediate_and_streams() {
  let temp = tempfile::tempdir().expect("tempdir");
  let root = AbsPath::try_from_std_path(temp.path()).expect("root");
  let dir = root.join("sub").expect("sub");
  let file = dir.join("stream.txt").expect("file");
  let fs = NativeFileSystem::new(false);

  WritableFileSystem::create_dir_all(&fs, &dir)
    .await
    .expect("create_dir_all");
  {
    let mut writer = IntermediateFileSystemExtras::create_write_stream(&fs, &file)
      .await
      .expect("create_write_stream");
    writer.write_line("hello").await.expect("write_line");
    writer.write_all(b"world").await.expect("write_all");
    writer.close().await.expect("close");
  }

  let mut reader = IntermediateFileSystemExtras::create_read_stream(&fs, &file)
    .await
    .expect("create_read_stream");
  assert_eq!(reader.read_line().await.expect("read_line"), "hello");
  assert_eq!(reader.read_to_end().await.expect("read_to_end"), b"world");

  let source = root.join("source").expect("source");
  let destination = root.join("destination").expect("destination");
  WritableFileSystem::create_dir(&fs, &source)
    .await
    .expect("mkdir source");
  WritableFileSystem::write(&fs, &source.join("content").expect("content"), b"source")
    .await
    .expect("write source");
  WritableFileSystem::create_dir(&fs, &destination)
    .await
    .expect("mkdir dest");
  WritableFileSystem::write(
    &fs,
    &destination.join("blocker").expect("blocker"),
    b"blocker",
  )
  .await
  .expect("write blocker");

  let error = IntermediateFileSystemExtras::rename(&fs, &source, &destination)
    .await
    .expect_err("rename onto non-empty dir should fail");
  let message = error.to_string();
  assert!(message.contains(source.as_str()));
  assert!(message.contains(destination.as_str()));

  WritableFileSystem::remove_dir_all(&fs, &dir)
    .await
    .expect("remove_dir_all");
  assert!(matches!(
    WritableFileSystem::stat(&fs, &dir).await,
    Err(Error::Io(e)) if e.kind() == ErrorKind::NotFound
  ));

  let as_intermediate: &dyn IntermediateFileSystem = &fs;
  assert!(matches!(
    as_intermediate.stat(&dir).await,
    Err(Error::Io(e)) if e.kind() == ErrorKind::NotFound
  ));
}

#[tokio::test]
async fn noop_fs_basics() {
  let root = if cfg!(windows) {
    AbsPath::new(r"C:\repo").expect("repo")
  } else {
    AbsPath::new("/repo").expect("repo")
  };
  let file_abs = root.join("dist/out.js").expect("out");

  let noop = NoopFileSystem;
  noop.create_dir_all(&root).await.expect("noop mkdir");
  noop.write(&file_abs, b"x").await.expect("noop write");
  noop
    .set_permissions(&file_abs, FilePermissions::from_mode(0o644))
    .await
    .expect("noop chmod");
  assert!(
    noop
      .read_dir(&root)
      .await
      .expect("noop read_dir")
      .is_empty()
  );
  assert!(matches!(
    noop.read_file(&file_abs).await,
    Err(Error::Io(e)) if e.kind() == ErrorKind::NotFound
  ));
  assert!(matches!(
    noop.stat(&file_abs).await,
    Err(Error::Io(e)) if e.kind() == ErrorKind::NotFound
  ));
}
