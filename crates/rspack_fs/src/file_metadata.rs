use std::fs::Metadata;

use crate::{Error, Result};

#[derive(Debug)]
pub struct FileMetadata {
  pub is_file: bool,
  pub is_directory: bool,
  pub is_symlink: bool,
  pub atime_ms: u64,
  pub mtime_ms: u64,
  pub ctime_ms: u64,
  pub size: u64,
}

impl TryFrom<Metadata> for FileMetadata {
  type Error = Error;

  fn try_from(metadata: Metadata) -> Result<Self> {
    let mtime_ms = metadata
      .modified()
      .map_err(Error::from)?
      .duration_since(std::time::UNIX_EPOCH)
      .expect("mtime is before unix epoch")
      .as_millis() as u64;
    let ctime_ms = metadata
      .created()
      .map_err(Error::from)?
      .duration_since(std::time::UNIX_EPOCH)
      .expect("ctime is before unix epoch")
      .as_millis() as u64;
    let atime_ms = metadata
      .accessed()
      .map_err(Error::from)?
      .duration_since(std::time::UNIX_EPOCH)
      .expect("atime is before unix epoch")
      .as_millis() as u64;
    Ok(Self {
      is_directory: metadata.is_dir(),
      is_file: metadata.is_file(),
      is_symlink: metadata.is_symlink(),
      size: metadata.len(),
      mtime_ms,
      ctime_ms,
      atime_ms,
    })
  }
}
