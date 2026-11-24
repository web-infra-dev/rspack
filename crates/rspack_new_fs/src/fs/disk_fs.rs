use std::sync::Arc;

use super::{FileMetadata, ReadableFileSystem, RootTable};
use crate::{AbsPath, ProjectPath, Result};

/// Root-aware read filesystem facade operating strictly on portable [`ProjectPath`]s.
#[derive(Debug, Clone)]
pub struct DiskFileSystem {
  fs: Arc<dyn ReadableFileSystem>,
  roots: Arc<RootTable>,
}

impl DiskFileSystem {
  pub fn new(fs: Arc<dyn ReadableFileSystem>, roots: Vec<(String, AbsPath)>) -> Result<Self> {
    let table = Arc::new(RootTable::from_fs(fs.as_ref(), roots)?);
    Ok(Self::from_root_table(fs, table))
  }

  pub fn from_root_table(fs: Arc<dyn ReadableFileSystem>, roots: Arc<RootTable>) -> Self {
    Self { fs, roots }
  }

  #[inline]
  pub fn roots(&self) -> &Arc<RootTable> {
    &self.roots
  }

  #[inline]
  pub fn backend(&self) -> &Arc<dyn ReadableFileSystem> {
    &self.fs
  }

  #[inline]
  pub fn to_project_path(&self, abs: &AbsPath) -> ProjectPath {
    self.roots.to_project_path(abs)
  }

  #[inline]
  pub fn to_project_path_str(&self, raw: &str) -> Result<ProjectPath> {
    self.roots.to_project_path_str(raw)
  }

  #[inline]
  pub fn to_abs_path(&self, path: &ProjectPath) -> Result<AbsPath> {
    self.roots.to_abs_path(path)
  }

  pub fn join(&self, base: &ProjectPath, sub: &str) -> Result<ProjectPath> {
    if let Some(joined) = base.try_join(sub) {
      return Ok(joined);
    }
    let abs = self.to_abs_path(base)?.join(sub)?;
    Ok(self.to_project_path(&abs))
  }

  pub fn parent(&self, path: &ProjectPath) -> Result<Option<ProjectPath>> {
    if let Some(parent) = path.parent_in_root() {
      return Ok(Some(parent));
    }
    let abs = self.to_abs_path(path)?;
    Ok(abs.parent().map(|parent| self.to_project_path(&parent)))
  }

  pub async fn read(&self, path: &ProjectPath) -> Result<Vec<u8>> {
    let abs = self.to_abs_path(path)?;
    self.fs.read(&abs).await
  }

  pub fn read_sync(&self, path: &ProjectPath) -> Result<Vec<u8>> {
    let abs = self.to_abs_path(path)?;
    self.fs.read_sync(&abs)
  }

  pub async fn metadata(&self, path: &ProjectPath) -> Result<FileMetadata> {
    let abs = self.to_abs_path(path)?;
    self.fs.metadata(&abs).await
  }

  pub fn metadata_sync(&self, path: &ProjectPath) -> Result<FileMetadata> {
    let abs = self.to_abs_path(path)?;
    self.fs.metadata_sync(&abs)
  }

  pub async fn symlink_metadata(&self, path: &ProjectPath) -> Result<FileMetadata> {
    let abs = self.to_abs_path(path)?;
    self.fs.symlink_metadata(&abs).await
  }

  pub async fn read_dir(&self, dir: &ProjectPath) -> Result<Vec<String>> {
    let abs = self.to_abs_path(dir)?;
    self.fs.read_dir(&abs).await
  }

  pub fn read_dir_sync(&self, dir: &ProjectPath) -> Result<Vec<String>> {
    let abs = self.to_abs_path(dir)?;
    self.fs.read_dir_sync(&abs)
  }

  pub async fn canonicalize(&self, path: &ProjectPath) -> Result<ProjectPath> {
    let abs = self.to_abs_path(path)?;
    let canon = self.fs.canonicalize(&abs).await?;
    Ok(self.to_project_path(&canon))
  }
}
