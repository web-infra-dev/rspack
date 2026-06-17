use std::{
  io::{self},
  sync::Arc,
};

use rspack_fs::{Error, FsResultToIoResultExt, ReadableFileSystem};
use rspack_paths::AssertUtf8;
use rspack_resolver::{FileMetadata, FileSystem as ResolverFileSystem};

#[derive(Clone)]
pub struct BoxFS(Arc<dyn ReadableFileSystem>);

impl BoxFS {
  pub fn new(fs: Arc<dyn ReadableFileSystem>) -> Self {
    Self(fs)
  }
}

// STAR-DIAG: temporary instrumentation to debug the flaky wasm-only resolve
// failure of `configCases/code-generation/path-ends-with-star`. Logs every
// filesystem call the resolver makes for that case, with the raw result/errno,
// so we can see what the wasm (wasi) fs actually returns when resolution fails.
#[inline]
fn star_diag_path(path: &std::path::Path) -> bool {
  path
    .to_str()
    .is_some_and(|p| p.contains("path-ends-with-star"))
}

// STAR-DIAG: widen the resolution window for the `star*` segment so the
// (suspected) cross-suite remove-during-resolve race reproduces in a single
// CI run instead of ~15% of the time. Sleeping *before* the underlying fs call
// gives the other parallel suite's `done`-hook `rmSync(star*)` time to land in
// the middle of this resolution.
#[inline]
fn star_diag_widen(path: &std::path::Path) {
  if path.to_str().is_some_and(|p| p.contains("star*")) {
    std::thread::sleep(std::time::Duration::from_millis(150));
  }
}

fn star_diag_meta(
  op: &str,
  path: &std::path::Path,
  res: &rspack_fs::Result<rspack_fs::FileMetadata>,
) {
  if !star_diag_path(path) {
    return;
  }
  let tid = std::thread::current().id();
  match res {
    Ok(m) => eprintln!(
      "[STAR-DIAG] {op}({}) -> Ok{{is_file:{}, is_dir:{}, is_symlink:{}}} tid={tid:?}",
      path.display(),
      m.is_file,
      m.is_directory,
      m.is_symlink
    ),
    Err(Error::Io(e)) => eprintln!(
      "[STAR-DIAG] {op}({}) -> Err{{kind:{:?}, errno:{:?}, msg:{}}} tid={tid:?}",
      path.display(),
      e.kind(),
      e.raw_os_error(),
      e
    ),
  }
}

#[async_trait::async_trait]
impl ResolverFileSystem for BoxFS {
  async fn read(&self, path: &std::path::Path) -> io::Result<Vec<u8>> {
    let res = self.0.read(path.assert_utf8()).await.to_io_result();
    if star_diag_path(path) {
      eprintln!(
        "[STAR-DIAG] read({}) -> {} bytes (tid={:?})",
        path.display(),
        res.as_ref().map(|b| b.len() as isize).unwrap_or(-1),
        std::thread::current().id()
      );
    }
    res
  }
  async fn read_to_string(&self, path: &std::path::Path) -> std::io::Result<String> {
    match self.0.read(path.assert_utf8()).await {
      Ok(x) => String::from_utf8(x).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err)),
      Err(Error::Io(e)) => Err(e),
    }
  }
  async fn metadata(&self, path: &std::path::Path) -> io::Result<FileMetadata> {
    star_diag_widen(path);
    let raw = self.0.metadata(path.assert_utf8()).await;
    star_diag_meta("metadata", path, &raw);
    match raw {
      Ok(meta) => Ok(FileMetadata {
        is_dir: meta.is_directory,
        is_file: meta.is_file,
        is_symlink: meta.is_symlink,
      }),
      Err(Error::Io(e)) => Err(e),
    }
  }

  async fn symlink_metadata(&self, path: &std::path::Path) -> io::Result<FileMetadata> {
    star_diag_widen(path);
    let raw = self.0.symlink_metadata(path.assert_utf8()).await;
    star_diag_meta("symlink_metadata", path, &raw);
    match raw {
      Ok(meta) => Ok(FileMetadata {
        is_dir: meta.is_directory,
        is_file: meta.is_file,
        is_symlink: meta.is_symlink,
      }),
      Err(Error::Io(e)) => Err(e),
    }
  }

  async fn canonicalize(&self, path: &std::path::Path) -> io::Result<std::path::PathBuf> {
    let raw = self.0.canonicalize(path.assert_utf8()).await;
    if star_diag_path(path) {
      let tid = std::thread::current().id();
      match &raw {
        Ok(p) => eprintln!(
          "[STAR-DIAG] canonicalize({}) -> Ok({}) tid={tid:?}",
          path.display(),
          p
        ),
        Err(Error::Io(e)) => eprintln!(
          "[STAR-DIAG] canonicalize({}) -> Err{{kind:{:?}, errno:{:?}, msg:{}}} tid={tid:?}",
          path.display(),
          e.kind(),
          e.raw_os_error(),
          e
        ),
      }
    }
    match raw {
      Ok(path) => Ok(path.into()),
      Err(Error::Io(e)) => Err(e),
    }
  }
}
