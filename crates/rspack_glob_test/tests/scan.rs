use std::{
  borrow::Cow,
  sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
  },
};

use rspack_fs::{
  FileMetadata, FilePermissions, MemoryFileSystem, ReadableFileSystem, WritableFileSystem,
};
use rspack_glob::{GlobOptions, GlobPattern};
use rspack_paths::{
  Utf8Path, Utf8PathBuf, normalize_native_path_separators, normalize_path_separators,
};

fn default_options() -> GlobOptions {
  GlobOptions {
    require_literal_leading_dot: true,
    ..Default::default()
  }
}

fn matches_with_options(pattern: &str, path: &str, options: &GlobOptions) -> bool {
  GlobPattern::new_with_options(pattern.as_bytes(), *options)
    .expect("valid glob")
    .match_path(path)
    .is_exact()
}
fn matches_with_explicit_dot(
  pattern: &str,
  path: &str,
  _base: &str,
  options: &GlobOptions,
) -> bool {
  GlobPattern::new_with_options(
    pattern.as_bytes(),
    GlobOptions {
      windows_paths: true,
      ..*options
    },
  )
  .expect("valid path glob")
  .match_path(normalize_path_separators(path).as_bytes())
  .is_exact()
}
fn pattern_has_explicit_dot_for(
  pattern: &str,
  _base: &str,
  path: &str,
  options: &GlobOptions,
) -> bool {
  matches_with_options(pattern, path, options)
}

#[derive(Debug, Default)]
struct ReadDirCountingFileSystem {
  inner: MemoryFileSystem,
  read_dir_count: AtomicUsize,
  visited: Mutex<Vec<Utf8PathBuf>>,
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
    ReadableFileSystem::read_dir(&self.inner, dir).await
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
async fn literal_pattern_does_not_walk_its_base_directory() {
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

  let entries = GlobPattern::new_with_options("/project/index.html".as_bytes(), default_options())
    .expect("valid glob")
    .scan(
      Utf8Path::new("."),
      fs.clone() as Arc<dyn ReadableFileSystem>,
    )
    .await
    .unwrap();

  assert_eq!(entries, vec![Utf8PathBuf::from("/project/index.html")]);
  assert_eq!(fs.read_dir_count.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn literal_pattern_returns_nothing_when_the_file_is_missing() {
  let fs = Arc::new(MemoryFileSystem::default());
  fs.create_dir_all("/project".into()).await.unwrap();

  let entries = GlobPattern::new_with_options("/project/index.html".as_bytes(), default_options())
    .expect("valid glob")
    .scan(Utf8Path::new("."), fs as Arc<dyn ReadableFileSystem>)
    .await
    .unwrap();

  assert!(entries.is_empty());
}

#[test]
fn directory_prefix_decodes_escapes_and_shared_branches() {
  for (source, expected) in [
    (r"./fixtures/a\[b\]/file", "./fixtures/a[b]/"),
    (r"./fixtures/a\[b\]/**/*.js", "./fixtures/a[b]/"),
    (r"./fixtures/file\*.js", "./fixtures/"),
    (
      r"./fixtures/directory\?1/**/*.js",
      "./fixtures/directory?1/",
    ),
    ("{src/a,src/b}/*.js", "src/"),
    ("./{src,src}/components/*.js", "./src/components/"),
    ("{src,lib}/*.js", ""),
    ("*.js", ""),
    ("!src/**/*.js", ""),
  ] {
    let pattern = GlobPattern::new(source.as_bytes()).unwrap();
    assert_eq!(pattern.directory_prefix(), expected.as_bytes(), "{source}");
    assert!(
      pattern.match_prefix(pattern.directory_prefix()).is_some(),
      "{source}"
    );
  }
}

#[test]
fn directory_prefix_borrows_the_cached_literal_prefix() {
  let source = b"./src/components/*.js";
  let pattern = GlobPattern::new(source).unwrap();
  assert_eq!(pattern.directory_prefix(), b"./src/components/");
  assert_eq!(pattern.directory_prefix().as_ptr(), source.as_ptr());

  let remaining = pattern.match_prefix(b"./src/").unwrap();
  assert_eq!(remaining.directory_prefix(), b"components/");
  assert_eq!(remaining.directory_prefix().as_ptr(), source[6..].as_ptr());
}

#[test]
fn glob_parser_handles_windows_separators_and_glob_escapes() {
  for (source, expected, path) in [
    (
      r"./fixtures/a\[b\]/*.js",
      r"./fixtures/a\[b\]/*.js",
      "./fixtures/a[b]/value.js",
    ),
    (
      r"./fixtures/file\*.js",
      r"./fixtures/file\*.js",
      "./fixtures/file*.js",
    ),
    (
      r"./fixtures/file\?.js",
      r"./fixtures/file\?.js",
      "./fixtures/file?.js",
    ),
    (
      r"C:\fixtures\a\[b\]\file.js",
      r"C:/fixtures/a\[b\]/file.js",
      "C:/fixtures/a[b]/file.js",
    ),
    (
      r"C:\repo\src/*.js",
      "C:/repo/src/*.js",
      "C:/repo/src/value.js",
    ),
    (
      r"C:\目录\文件\[name\]/*.js",
      r"C:/目录/文件\[name\]/*.js",
      "C:/目录/文件[name]/value.js",
    ),
  ] {
    let pattern = GlobPattern::new_with_options(
      source.as_bytes(),
      GlobOptions {
        windows_paths: true,
        ..Default::default()
      },
    )
    .unwrap();
    assert_eq!(pattern.source(), expected.as_bytes(), "{source}");
    assert!(pattern.match_path(path).is_exact(), "{source}");
    let remaining = pattern.match_prefix(pattern.directory_prefix()).unwrap();
    let suffix = path
      .as_bytes()
      .strip_prefix(pattern.directory_prefix())
      .unwrap();
    assert!(remaining.match_path(suffix).is_exact(), "{source}");
  }
}

#[test]
fn normalize_path_separators_treats_glob_chars_as_literals() {
  assert_eq!(
    normalize_path_separators("C:\\fixtures\\a\\[b]\\file.js"),
    "C:/fixtures/a/[b]/file.js"
  );
  assert_eq!(
    normalize_path_separators("C:\\fixtures\\a\\{b}\\file.js"),
    "C:/fixtures/a/{b}/file.js"
  );
}

#[test]
fn separator_normalization_borrows_unchanged_inputs() {
  let path = "./src/目录/index.js";
  let normalized = normalize_path_separators(path);
  assert!(matches!(normalized, Cow::Borrowed(_)));
  assert_eq!(normalized.as_ptr(), path.as_ptr());

  let pattern = r"./src/目录/\[literal\]/file\*.js";
  let parsed = GlobPattern::new_with_options(
    pattern.as_bytes(),
    GlobOptions {
      windows_paths: true,
      ..Default::default()
    },
  )
  .unwrap();
  assert_eq!(parsed.source(), pattern.as_bytes());
  assert_eq!(parsed.source().as_ptr(), pattern.as_ptr());
}

#[test]
fn native_separator_normalization_preserves_unix_filename_backslashes() {
  let path = r"src\literal\*.js";
  let normalized = normalize_native_path_separators(path);
  if cfg!(windows) {
    assert_eq!(normalized, "src/literal/*.js");
  } else {
    assert_eq!(normalized, path);
    assert!(matches!(normalized, Cow::Borrowed(_)));
  }
}

#[test]
fn recursive_matching_uses_remaining_states() {
  for (source, expected) in [
    ("./src/*.js", false),
    ("./src/value.js", false),
    (r"./src/file\*\*.js", false),
    ("./src/{a,b}.js", false),
    ("./{src,src}/*.js", false),
    ("./src/**/*.js", true),
    ("./src/{a,b/nested}.js", true),
    ("{src/a,src/b}/*.js", true),
    ("!*.js", true),
  ] {
    let pattern = GlobPattern::new(source.as_bytes()).unwrap();
    let remaining = pattern.match_prefix(pattern.directory_prefix()).unwrap();
    assert_eq!(remaining.is_recursive(), expected, "{source}");
  }

  let pattern = GlobPattern::new(b"src/{a,b/nested}.js").unwrap();
  let remaining = pattern.match_prefix(b"src/b/").unwrap();
  assert!(!remaining.is_recursive());
  assert!(remaining.match_path(b"nested.js").is_exact());
}

#[test]
fn escaped_star_and_question_match_literal_path_segments() {
  let options = default_options();

  assert!(matches_with_options(
    "./fixtures/file\\*.js",
    "./fixtures/file*.js",
    &options
  ));
  assert!(!matches_with_options(
    "./fixtures/file\\*.js",
    "./fixtures/file-a.js",
    &options
  ));
  assert!(matches_with_options(
    "./fixtures/directory\\?1/**/*.js",
    "./fixtures/directory?1/index.js",
    &options
  ));
  assert!(!matches_with_options(
    "./fixtures/directory\\?1/**/*.js",
    "./fixtures/directory-a1/index.js",
    &options
  ));
}

#[test]
fn explicit_dot_patterns_allow_wildcard_dot_segments() {
  let base_dir = "./fixtures/";
  let options = default_options();

  assert!(pattern_has_explicit_dot_for(
    "./fixtures/**/.*",
    base_dir,
    "./fixtures/.env",
    &options
  ));
  assert!(pattern_has_explicit_dot_for(
    "./fixtures/**/.*/index.js",
    base_dir,
    "./fixtures/.cache/index.js",
    &options
  ));
  assert!(!pattern_has_explicit_dot_for(
    "./fixtures/**/index.js",
    base_dir,
    "./fixtures/.cache/index.js",
    &options
  ));
}

#[test]
fn explicit_dot_patterns_respect_case_insensitive_matching() {
  let base_dir = "./fixtures/";
  let options = GlobOptions {
    case_sensitive: false,
    ..Default::default()
  };

  assert!(pattern_has_explicit_dot_for(
    "./fixtures/**/.ENV",
    base_dir,
    "./fixtures/.env",
    &options
  ));
}

#[test]
fn matches_with_explicit_dot_treats_windows_path_separators_as_separators() {
  let options = default_options();
  assert!(matches_with_explicit_dot(
    "C:/repo/escape/**/glob.js",
    "C:\\repo\\escape\\[brackets]\\glob.js",
    "C:/repo/escape/",
    &options
  ));
  assert!(matches_with_explicit_dot(
    "C:/repo/escape/**/glob.js",
    "C:\\repo\\escape\\{curlies}\\glob.js",
    "C:/repo/escape/",
    &options
  ));
}

#[test]
fn matches_with_explicit_dot_requires_literal_dot_segments() {
  let options = default_options();
  assert!(matches_with_explicit_dot(
    "./fixtures/.*.js",
    "./fixtures/.hidden.js",
    "./fixtures/",
    &options
  ));
  assert!(!matches_with_explicit_dot(
    "./fixtures/*.js",
    "./fixtures/.hidden.js",
    "./fixtures/",
    &options
  ));
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
  let mut files = pattern.scan(root, fs).await.expect("scan");
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
  let mut files = pattern
    .scan(Utf8Path::new("/project"), fs.clone())
    .await
    .expect("scan");
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
    visited.len(),
    2,
    "only the two dynamic filename directories are read"
  );
  assert!(!visited.iter().any(|path| path == Utf8Path::new("/project")));
  assert!(
    !visited
      .iter()
      .any(|path| path.ends_with("unrelated") || path.ends_with("nested"))
  );
}

#[tokio::test]
async fn scan_literal_branches_uses_no_directory_reads() {
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
  assert_eq!(pattern.scan_root(root), root.join("src"));
  let mut files = pattern.scan(root, fs.clone()).await.unwrap();
  files.sort();
  assert_eq!(
    files,
    vec![root.join("src/one.js"), root.join("src/two.js")]
  );
  assert_eq!(fs.read_dir_count.load(Ordering::Relaxed), 0);
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
  assert!(pattern.match_path("nested").is_exact());
  let mut files = pattern.walk(Utf8Path::new("/project"), fs).await.unwrap();
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
    pattern
      .scan(Utf8Path::new("/project"), fs)
      .await
      .expect("scan"),
    vec![Utf8PathBuf::from("/project/assets/value.js")]
  );
}
