use std::sync::{
  Arc, Mutex,
  atomic::{AtomicUsize, Ordering},
};

use rspack_fs::{
  FileMetadata, FilePermissions, MemoryFileSystem, ReadableFileSystem, WalkMatcher,
  WritableFileSystem,
};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::glob::{BackslashMode, GlobOptions, GlobPattern};

async fn walk_matches(
  pattern: &GlobPattern,
  root: &Utf8Path,
  fs: Arc<dyn ReadableFileSystem>,
) -> Vec<Utf8PathBuf> {
  let mut files = Vec::new();
  pattern
    .walk(root, fs.as_ref(), false, &mut |path, _| files.push(path))
    .await
    .expect("walk matching paths");
  files
}

async fn scan_matches(
  pattern: &GlobPattern,
  context: &Utf8Path,
  fs: Arc<dyn ReadableFileSystem>,
) -> Vec<Utf8PathBuf> {
  let prefix = pattern.directory_prefix();
  let root = context.join(std::str::from_utf8(&prefix).expect("UTF-8 directory prefix"));
  let remaining = pattern.match_prefix(&prefix).expect("valid literal prefix");
  walk_matches(&remaining, &root, fs).await
}

fn default_options() -> GlobOptions {
  GlobOptions {
    require_literal_leading_dot: true,
    ..Default::default()
  }
}

#[derive(Debug, Default)]
struct ReadDirCountingFileSystem {
  inner: MemoryFileSystem,
  read_dir_count: AtomicUsize,
  visited: Mutex<Vec<Utf8PathBuf>>,
  metadata_paths: Mutex<Vec<Utf8PathBuf>>,
  missing_metadata: Option<Utf8PathBuf>,
  metadata_alias: Option<(Utf8PathBuf, Utf8PathBuf)>,
}

#[async_trait::async_trait]
impl ReadableFileSystem for ReadDirCountingFileSystem {
  async fn read(&self, path: &Utf8Path) -> rspack_fs::Result<Vec<u8>> {
    self.inner.read(path).await
  }

  fn read_sync(&self, path: &Utf8Path) -> rspack_fs::Result<Vec<u8>> {
    self.inner.read_sync(path)
  }

  async fn metadata(&self, path: &Utf8Path) -> rspack_fs::Result<FileMetadata> {
    self
      .metadata_paths
      .lock()
      .expect("metadata paths lock")
      .push(path.to_path_buf());
    if self.missing_metadata.as_deref() == Some(path) {
      return Err(rspack_fs::Error::new(
        std::io::ErrorKind::NotFound,
        "entry disappeared after read_dir",
      ));
    }
    if let Some((alias, actual)) = &self.metadata_alias
      && alias == path
    {
      return self.inner.metadata(actual).await;
    }
    self.inner.metadata(path).await
  }

  fn metadata_sync(&self, path: &Utf8Path) -> rspack_fs::Result<FileMetadata> {
    self.inner.metadata_sync(path)
  }

  async fn symlink_metadata(&self, path: &Utf8Path) -> rspack_fs::Result<FileMetadata> {
    self.inner.symlink_metadata(path).await
  }

  async fn read_link(&self, path: &Utf8Path) -> rspack_fs::Result<Utf8PathBuf> {
    self.inner.read_link(path).await
  }

  async fn canonicalize(&self, path: &Utf8Path) -> rspack_fs::Result<Utf8PathBuf> {
    self.inner.canonicalize(path).await
  }

  async fn read_dir(&self, dir: &Utf8Path) -> rspack_fs::Result<Vec<String>> {
    self.read_dir_count.fetch_add(1, Ordering::Relaxed);
    self
      .visited
      .lock()
      .expect("visited directories lock")
      .push(dir.to_path_buf());
    let mut names = ReadableFileSystem::read_dir(&self.inner, dir).await?;
    names.sort_unstable();
    Ok(names)
  }

  fn read_dir_sync(&self, dir: &Utf8Path) -> rspack_fs::Result<Vec<String>> {
    self.read_dir_count.fetch_add(1, Ordering::Relaxed);
    self
      .visited
      .lock()
      .expect("visited directories lock")
      .push(dir.to_path_buf());
    self.inner.read_dir_sync(dir)
  }

  async fn permissions(&self, path: &Utf8Path) -> rspack_fs::Result<Option<FilePermissions>> {
    self.inner.permissions(path).await
  }
}

#[tokio::test]
async fn walkers_preserve_their_metadata_error_policy() {
  #[derive(Clone)]
  struct AllEntries;

  impl WalkMatcher for AllEntries {
    const VISIT_ON_METADATA_ERROR: bool = true;
  }

  let root = Utf8Path::new("/project");
  let entry = root.join("entry.js");
  let fs = Arc::new(ReadDirCountingFileSystem {
    missing_metadata: Some(entry.clone()),
    ..Default::default()
  });
  fs.inner.create_dir_all(root).await.unwrap();
  fs.inner.write(&entry, b"").await.unwrap();

  let mut files = Vec::new();
  AllEntries
    .walk(root, fs.as_ref(), false, &mut |path, _| {
      files.push(path);
    })
    .await
    .unwrap();
  assert_eq!(files.as_slice(), std::slice::from_ref(&entry));

  let glob = GlobPattern::new(b"*.js").unwrap();
  assert_eq!(walk_matches(&glob, root, fs).await, [entry]);
}

#[tokio::test]
async fn literal_pattern_only_reads_its_base_directory() {
  let fs = Arc::new(ReadDirCountingFileSystem::default());
  fs.inner
    .create_dir_all("/project/deps".into())
    .await
    .unwrap();
  fs.inner
    .write("/project/index.html".into(), "abc".as_bytes())
    .await
    .unwrap();
  fs.inner
    .write("/project/deps/other.html".into(), "abc".as_bytes())
    .await
    .unwrap();

  let pattern = GlobPattern::new_with_options("/project/index.html".as_bytes(), default_options())
    .expect("valid glob");
  let entries = scan_matches(&pattern, Utf8Path::new("."), fs.clone()).await;

  assert_eq!(entries, vec![Utf8PathBuf::from("/project/index.html")]);
  assert_eq!(*fs.visited.lock().unwrap(), [Utf8PathBuf::from("/project")]);
}

#[tokio::test]
async fn literal_pattern_returns_nothing_when_the_file_is_missing() {
  let fs = Arc::new(MemoryFileSystem::default());
  fs.create_dir_all("/project".into()).await.unwrap();

  let pattern = GlobPattern::new_with_options("/project/index.html".as_bytes(), default_options())
    .expect("valid glob");
  let entries = scan_matches(&pattern, Utf8Path::new("."), fs).await;

  assert!(entries.is_empty());
}

#[tokio::test]
async fn native_windows_patterns_scan_from_the_literal_directory_prefix() {
  let fs = Arc::new(ReadDirCountingFileSystem::default());
  let root = Utf8Path::new("/project");
  fs.inner
    .create_dir_all(root.join("src").as_path())
    .await
    .unwrap();
  fs.inner
    .write(root.join("src/value.txt").as_path(), b"value")
    .await
    .unwrap();
  let pattern = GlobPattern::new_with_options(
    br"src\*.txt",
    GlobOptions {
      backslash: BackslashMode::Separator,
      ..Default::default()
    },
  )
  .unwrap();
  assert_eq!(pattern.directory_prefix(), b"src/");
  assert_eq!(
    scan_matches(&pattern, root, fs.clone()).await,
    vec![root.join("src/value.txt")]
  );
  assert_eq!(*fs.visited.lock().unwrap(), vec![root.join("src")]);
}

#[tokio::test]
async fn scan_keeps_physical_metacharacters_out_of_pattern_syntax() {
  let fs = Arc::new(MemoryFileSystem::default());
  let root = Utf8Path::new("/project/[src]/{assets}");
  fs.create_dir_all(&root.join(".cache"))
    .await
    .expect("create directory");
  for filename in ["index.js", ".env", ".cache/value.js", "index.css"] {
    fs.write(root.join(filename).as_path(), b"value")
      .await
      .expect("write fixture");
  }
  let pattern = GlobPattern::new_with_options("**/{*.js,.env}".as_bytes(), default_options())
    .expect("valid glob");
  let mut files = scan_matches(&pattern, root, fs).await;
  files.sort();
  assert_eq!(files, vec![root.join(".env"), root.join("index.js")]);
}

#[tokio::test]
async fn scan_prunes_unrelated_directories_and_exhausted_patterns() {
  let fs = Arc::new(ReadDirCountingFileSystem::default());
  for dir in ["components/nested", "widgets", "unrelated"] {
    fs.inner
      .create_dir_all(Utf8Path::new("/project").join(dir).as_path())
      .await
      .expect("create directory");
  }
  for filename in [
    "components/value.js",
    "widgets/value.js",
    "components/nested/unused.js",
    "unrelated/unused.js",
  ] {
    fs.inner
      .write(Utf8Path::new("/project").join(filename).as_path(), b"value")
      .await
      .expect("write fixture");
  }
  let pattern =
    GlobPattern::new_with_options("{components,widgets}/*.js".as_bytes(), default_options())
      .expect("valid glob");
  let mut files = scan_matches(&pattern, Utf8Path::new("/project"), fs.clone()).await;
  files.sort();
  assert_eq!(
    files,
    vec![
      Utf8PathBuf::from("/project/components/value.js"),
      Utf8PathBuf::from("/project/widgets/value.js")
    ]
  );
  let visited = fs.visited.lock().expect("visited directories lock");
  assert_eq!(
    *visited,
    ["/project", "/project/components", "/project/widgets"].map(Utf8PathBuf::from)
  );
}

#[tokio::test]
async fn scan_literal_branches_reuses_a_directory_listing() {
  let fs = Arc::new(ReadDirCountingFileSystem::default());
  let root = Utf8Path::new("/project/[physical]");
  fs.inner
    .create_dir_all(root.join("src").as_path())
    .await
    .unwrap();
  for name in ["one.js", "two.js", "other.js"] {
    fs.inner
      .write(root.join("src").join(name).as_path(), b"value")
      .await
      .unwrap();
  }
  let pattern = GlobPattern::new("{src/one.js,src/two.js,src/missing.js}".as_bytes()).unwrap();
  assert_eq!(pattern.directory_prefix(), b"src/");
  let mut files = scan_matches(&pattern, root, fs.clone()).await;
  files.sort();
  assert_eq!(
    files,
    vec![root.join("src/one.js"), root.join("src/two.js")]
  );
  assert_eq!(fs.read_dir_count.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn scan_matches_case_against_enumerated_entry_names() {
  let root = Utf8Path::new("/project");
  let fs = Arc::new(ReadDirCountingFileSystem {
    metadata_alias: Some((root.join("other.js"), root.join("OTHER.js"))),
    ..Default::default()
  });
  fs.inner.create_dir_all(root).await.unwrap();
  for name in ["one.js", "OTHER.js"] {
    fs.inner.write(&root.join(name), b"").await.unwrap();
  }

  let pattern = GlobPattern::new(b"{one,other}.js").unwrap();
  assert_eq!(
    walk_matches(&pattern, root, fs).await,
    [root.join("one.js")]
  );
}

#[tokio::test]
async fn walk_preserves_depth_first_order_and_sibling_patterns() {
  let fs = Arc::new(ReadDirCountingFileSystem::default());
  let root = Utf8Path::new("/project");
  for directory in ["a/deep", "z"] {
    fs.inner
      .create_dir_all(root.join(directory).as_path())
      .await
      .expect("create directory");
  }
  let expected = ["a/deep/first.js", "a/last.js", "z/other.js"].map(|name| root.join(name));
  for path in &expected {
    fs.inner.write(path, b"value").await.expect("write fixture");
  }
  let pattern = GlobPattern::new(b"{a/deep/first.js,a/last.js,z/other.js}").expect("valid glob");
  assert_eq!(walk_matches(&pattern, root, fs.clone()).await, expected);
  assert_eq!(fs.read_dir_count.load(Ordering::Relaxed), 4);
}

#[tokio::test]
async fn walk_keeps_globstar_continuations_after_exact_matches() {
  let fs = Arc::new(MemoryFileSystem::default());
  fs.create_dir_all("/project/nested".into()).await.unwrap();
  fs.write("/project/value.js".into(), b"value")
    .await
    .unwrap();
  fs.write("/project/nested/value.js".into(), b"value")
    .await
    .unwrap();
  let pattern = GlobPattern::new("**".as_bytes()).unwrap();
  assert!(pattern.is_match("nested"));
  let mut files = walk_matches(&pattern, Utf8Path::new("/project"), fs).await;
  files.sort();
  assert_eq!(
    files,
    vec![
      Utf8PathBuf::from("/project/nested/value.js"),
      Utf8PathBuf::from("/project/value.js")
    ]
  );
}

#[tokio::test]
async fn scan_case_insensitively_preserves_filesystem_spelling() {
  let fs = Arc::new(MemoryFileSystem::default());
  fs.create_dir_all("/project/assets".into())
    .await
    .expect("create directory");
  fs.write("/project/assets/value.js".into(), b"value")
    .await
    .expect("write fixture");
  let pattern = GlobPattern::new_with_options(
    "ASSETS/*.JS".as_bytes(),
    GlobOptions {
      case_sensitive: false,
      ..Default::default()
    },
  )
  .expect("valid scan");
  assert_eq!(
    walk_matches(&pattern, Utf8Path::new("/project"), fs).await,
    vec![Utf8PathBuf::from("/project/assets/value.js")]
  );
}

#[tokio::test]
async fn scan_prunes_non_matching_wildcard_entries_before_metadata() {
  let fs = Arc::new(ReadDirCountingFileSystem::default());
  let root = Utf8Path::new("/project");
  for dir in ["node_modules", "src"] {
    fs.inner
      .create_dir_all(root.join(dir).as_path())
      .await
      .unwrap();
  }
  for file in [
    "index.js",
    "README.md",
    "index.css",
    "index",
    "node_modules/pkg.js",
    "src/nested.js",
  ] {
    fs.inner
      .write(root.join(file).as_path(), b"value")
      .await
      .unwrap();
  }

  let pattern = GlobPattern::new_with_options(b"*.js", default_options()).unwrap();
  assert_eq!(
    scan_matches(&pattern, root, fs.clone()).await,
    vec![root.join("index.js")]
  );
  assert_eq!(*fs.visited.lock().unwrap(), vec![root.to_path_buf()]);
  assert_eq!(
    *fs.metadata_paths.lock().unwrap(),
    vec![root.to_path_buf(), root.join("index.js")]
  );

  fs.visited.lock().unwrap().clear();
  fs.metadata_paths.lock().unwrap().clear();

  let sub_pattern = GlobPattern::new_with_options(b"src/*.js", default_options()).unwrap();
  assert_eq!(
    walk_matches(&sub_pattern, root, fs.clone()).await,
    vec![root.join("src/nested.js")]
  );
  assert_eq!(
    *fs.visited.lock().unwrap(),
    vec![root.to_path_buf(), root.join("src")]
  );
  assert_eq!(
    *fs.metadata_paths.lock().unwrap(),
    vec![
      root.to_path_buf(),
      root.join("src"),
      root.join("src"),
      root.join("src/nested.js"),
    ]
  );
}
