#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use rspack_fs::ReadableFileSystem;
#[cfg(test)]
use rspack_glob::pattern_has_explicit_dot_for;
pub use rspack_glob::{
  GlobMatchOptions, escape_glob_pattern, extract_glob_base_dir, find_files_by_glob,
  glob_base_dir_end, glob_match_with_explicit_dot, glob_match_with_options, is_glob_metacharacter,
  normalize_path_separators, normalize_path_separators_for_path, unescape_glob_path,
};
pub(crate) use rspack_glob::{glob_match_normalized_with_explicit_dot, walk_dir};
#[cfg(test)]
use rspack_paths::{Utf8Path, Utf8PathBuf};

#[cfg(test)]
mod tests {
  use std::sync::atomic::{AtomicUsize, Ordering};

  use rspack_fs::{FileMetadata, FilePermissions, MemoryFileSystem, WritableFileSystem};

  use super::*;

  #[derive(Debug, Default)]
  struct ReadDirCountingFileSystem {
    inner: MemoryFileSystem,
    read_dir_count: AtomicUsize,
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
      ReadableFileSystem::read_dir(&self.inner, dir).await
    }

    fn read_dir_sync(&self, dir: &Utf8Path) -> rspack_fs::Result<Vec<String>> {
      self.read_dir_count.fetch_add(1, Ordering::Relaxed);
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

    let entries = find_files_by_glob(
      "/project/index.html",
      &GlobMatchOptions::default(),
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

    let entries = find_files_by_glob(
      "/project/index.html",
      &GlobMatchOptions::default(),
      fs as Arc<dyn ReadableFileSystem>,
    )
    .await
    .unwrap();

    assert!(entries.is_empty());
  }

  #[test]
  fn extract_glob_base_dir_skips_escaped_metacharacters() {
    assert_eq!(
      extract_glob_base_dir("./fixtures/a\\[b\\]/file"),
      "./fixtures/a\\[b\\]/"
    );
    assert_eq!(
      extract_glob_base_dir("./fixtures/a\\[b\\]/**/*.js"),
      "./fixtures/a\\[b\\]/"
    );
    assert_eq!(
      extract_glob_base_dir("./fixtures/file\\*.js"),
      "./fixtures/"
    );
    assert_eq!(
      extract_glob_base_dir("./fixtures/directory\\?1/**/*.js"),
      "./fixtures/directory\\?1/"
    );
  }

  #[test]
  fn normalize_path_separators_preserves_glob_escapes() {
    assert_eq!(
      normalize_path_separators("./fixtures/a\\[b\\]/**/*.js"),
      "./fixtures/a\\[b\\]/**/*.js"
    );
    assert_eq!(
      normalize_path_separators("./fixtures/file\\*.js"),
      "./fixtures/file\\*.js"
    );
    assert_eq!(
      normalize_path_separators("./fixtures/file\\?.js"),
      "./fixtures/file\\?.js"
    );
    assert_eq!(
      normalize_path_separators("C:\\fixtures\\a\\[b\\]\\file.js"),
      "C:/fixtures/a\\[b\\]/file.js"
    );
    assert_eq!(
      normalize_path_separators("C:\\repo\\src/*.js"),
      "C:/repo/src/*.js"
    );
  }

  #[test]
  fn normalize_path_separators_for_path_treats_glob_chars_as_literals() {
    assert_eq!(
      normalize_path_separators_for_path("C:\\fixtures\\a\\[b]\\file.js"),
      "C:/fixtures/a/[b]/file.js"
    );
    assert_eq!(
      normalize_path_separators_for_path("C:\\fixtures\\a\\{b}\\file.js"),
      "C:/fixtures/a/{b}/file.js"
    );
  }

  #[test]
  fn unescape_glob_path_restores_literal_path_segments() {
    assert_eq!(
      unescape_glob_path("./fixtures/a\\[b\\]/"),
      "./fixtures/a[b]/"
    );
    assert_eq!(
      unescape_glob_path("./fixtures/file\\*.js"),
      "./fixtures/file*.js"
    );
    assert_eq!(
      unescape_glob_path("./fixtures/directory\\?1/"),
      "./fixtures/directory?1/"
    );
  }

  #[test]
  fn escaped_star_and_question_match_literal_path_segments() {
    let options = GlobMatchOptions::default();

    assert!(glob_match_with_options(
      "./fixtures/file\\*.js",
      "./fixtures/file*.js",
      &options
    ));
    assert!(!glob_match_with_options(
      "./fixtures/file\\*.js",
      "./fixtures/file-a.js",
      &options
    ));
    assert!(glob_match_with_options(
      "./fixtures/directory\\?1/**/*.js",
      "./fixtures/directory?1/index.js",
      &options
    ));
    assert!(!glob_match_with_options(
      "./fixtures/directory\\?1/**/*.js",
      "./fixtures/directory-a1/index.js",
      &options
    ));
  }

  #[test]
  fn explicit_dot_patterns_allow_wildcard_dot_segments() {
    let base_dir = "./fixtures/";
    let options = GlobMatchOptions::default();

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
    let options = GlobMatchOptions {
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
  fn glob_match_with_explicit_dot_treats_windows_path_separators_as_separators() {
    let options = GlobMatchOptions::default();
    assert!(glob_match_with_explicit_dot(
      "C:/repo/escape/**/glob.js",
      "C:\\repo\\escape\\[brackets]\\glob.js",
      "C:/repo/escape/",
      &options
    ));
    assert!(glob_match_with_explicit_dot(
      "C:/repo/escape/**/glob.js",
      "C:\\repo\\escape\\{curlies}\\glob.js",
      "C:/repo/escape/",
      &options
    ));
  }

  #[test]
  fn glob_match_with_explicit_dot_requires_literal_dot_segments() {
    let options = GlobMatchOptions::default();
    assert!(glob_match_with_explicit_dot(
      "./fixtures/.*.js",
      "./fixtures/.hidden.js",
      "./fixtures/",
      &options
    ));
    assert!(!glob_match_with_explicit_dot(
      "./fixtures/*.js",
      "./fixtures/.hidden.js",
      "./fixtures/",
      &options
    ));
  }
}
