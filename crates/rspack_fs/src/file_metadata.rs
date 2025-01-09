use std::{fs::Metadata, io::ErrorKind};

use crate::{Error, Result};

#[derive(Debug, Clone)]
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
    let ctime_ms = match metadata.created() {
      Ok(time) => time
        .duration_since(std::time::UNIX_EPOCH)
        .expect("ctime is before unix epoch")
        .as_millis() as u64,
      Err(err) => {
        // some linux musl not support get create time
        // return 0 directly to solve this problem
        if err.kind() == ErrorKind::Unsupported {
          0_u64
        } else {
          return Err(err.into());
        }
      }
    };
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
