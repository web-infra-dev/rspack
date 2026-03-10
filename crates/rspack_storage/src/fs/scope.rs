use std::path::{Path, PathBuf};

use rspack_fs::FileMetadata;
use rspack_paths::AssertUtf8;

use super::{FSResult, FileSystem, Reader, Writer};

#[derive(Debug, Clone)]
pub struct ScopeFileSystem {
  workspace: PathBuf,
  fs: FileSystem,
}

impl std::fmt::Display for ScopeFileSystem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(&self.workspace.to_string_lossy(), f)
  }
}

impl ScopeFileSystem {
  #[cfg(test)]
  pub fn new_memory_fs(workspace: PathBuf) -> Self {
    Self {
      workspace,
      fs: FileSystem(std::sync::Arc::new(rspack_fs::MemoryFileSystem::default())),
    }
  }

  pub fn new(workspace: PathBuf, fs: FileSystem) -> Self {
    Self { workspace, fs }
  }

  pub async fn ensure_exist(&self) -> FSResult<()> {
    self
      .fs
      .ensure_dir(&self.workspace.clone().assert_utf8())
      .await?;
    Ok(())
  }

  pub async fn remove(&self) -> FSResult<()> {
    let path = self.workspace.clone().assert_utf8();
    self.fs.remove_dir(&path).await?;
    Ok(())
  }

  pub async fn move_to(
    from: ScopeFileSystem,
    to: ScopeFileSystem,
    relative_path: impl AsRef<Path>,
  ) -> FSResult<()> {
    let from_file = from.workspace.join(relative_path.as_ref());
    let to_file = to.workspace.join(relative_path.as_ref());
    from
      .fs
      .move_file(&from_file.assert_utf8(), &to_file.assert_utf8())
      .await?;

    Ok(())
  }

  pub fn child_fs(&self, relative_path: impl AsRef<Path>) -> Self {
    let workspace = self.workspace.join(relative_path);
    Self {
      workspace,
      fs: self.fs.clone(),
    }
  }

  pub async fn stat(&self, relative_path: impl AsRef<Path>) -> FSResult<FileMetadata> {
    let path = self.workspace.join(relative_path);
    self.fs.stat(&path.assert_utf8()).await
  }

  pub async fn remove_file(&self, relative_path: impl AsRef<Path>) -> FSResult<()> {
    let path = self.workspace.join(relative_path);
    self.fs.remove_file(&path.assert_utf8()).await
  }

  pub async fn write(&self, relative_path: impl AsRef<Path>, bytes: &[u8]) -> FSResult<()> {
    let path = self.workspace.join(relative_path);
    self.fs.write(&path.assert_utf8(), bytes).await
  }

  pub async fn read(&self, relative_path: impl AsRef<Path>) -> FSResult<Vec<u8>> {
    let path = self.workspace.join(relative_path);
    self.fs.read_file_bytes(&path.assert_utf8()).await
  }

  pub async fn read_file(&self, relative_path: impl AsRef<Path>) -> FSResult<Reader> {
    let path = self.workspace.join(relative_path);
    self.fs.read_file(&path.assert_utf8()).await
  }

  pub async fn write_file(&self, relative_path: impl AsRef<Path>) -> FSResult<Writer> {
    let path = self.workspace.join(relative_path);
    self.fs.write_file(&path.assert_utf8()).await
  }

  pub async fn list_child(&self) -> FSResult<rustc_hash::FxHashSet<String>> {
    self
      .fs
      .read_dir(&self.workspace.clone().assert_utf8())
      .await
  }
}
