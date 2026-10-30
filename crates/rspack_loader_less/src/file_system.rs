use std::{
  io,
  path::{Path, PathBuf},
  sync::Arc,
};

use parking_lot::Mutex;
use rspack_fs::{FsResultToIoResultExt, ReadableFileSystem};
use rspack_loader_runner::LoaderDependencies;
use rspack_paths::{AssertUtf8, Utf8PathBuf};
use sugar_path::SugarPath;

pub(crate) struct FileSystem {
  pub fs: Arc<dyn ReadableFileSystem>,
  pub runtime: tokio::runtime::Handle,
  pub context: Utf8PathBuf,
  pub dependencies: Arc<Mutex<LoaderDependencies>>,
}

impl FileSystem {
  fn path(&self, path: &Path) -> Utf8PathBuf {
    self
      .context
      .join(path.assert_utf8())
      .as_std_path()
      .normalize()
      .assert_utf8()
      .to_path_buf()
  }

  fn record(&self, path: &rspack_paths::Utf8Path, exists: bool) {
    let mut dependencies = self.dependencies.lock();
    if exists {
      dependencies.file.insert(path.into());
    } else {
      dependencies.missing.insert(path.into());
    }
  }
}

impl less_rs::FileSystem for FileSystem {
  fn read(&self, path: &Path) -> io::Result<String> {
    let bytes = self.read_bytes(path)?;
    String::from_utf8(bytes).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
  }

  fn read_bytes(&self, path: &Path) -> io::Result<Vec<u8>> {
    let path = self.path(path);
    let result = self.fs.read_sync(&path).to_io_result();
    // Retain dependencies even when a later import or evaluation fails.
    self.record(
      &path,
      !matches!(&result, Err(error) if error.kind() == io::ErrorKind::NotFound),
    );
    result
  }

  fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
    let path = self.path(path);
    // Less runs on Rayon, so the input file system can call back into Node.
    let resolved = self
      .runtime
      .block_on(self.fs.canonicalize(&path))
      .to_io_result()?;
    self.record(&path, true);
    self.record(&resolved, true);
    Ok(resolved.into_std_path_buf())
  }

  fn exists(&self, path: &Path) -> bool {
    let path = self.path(path);
    let exists = self
      .fs
      .metadata_sync(&path)
      .is_ok_and(|metadata| metadata.is_file);
    self.record(&path, exists);
    exists
  }
}
