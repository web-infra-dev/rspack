use std::{
  fs::{Metadata, Permissions},
  io::ErrorKind,
};

use cfg_if::cfg_if;

use crate::{Error, IoResultToFsResultExt, Result};

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
      .to_fs_result()?
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
      .to_fs_result()?
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

/// This is a target-agnostic file permission abstraction.
/// Currently we only support getting and setting file permissions on unix target.
/// If we are supporting more targets, organizing the code like [std::sys::fs] will be a better choice.
#[derive(Debug, Clone)]
pub struct FilePermissions(#[cfg(target_family = "unix")] u32);

impl FilePermissions {
  cfg_if! {
      if #[cfg(target_family = "unix")] {
        pub fn from_mode(mode: u32) -> Self {
          Self(mode)
        }

        pub fn into_mode(self) -> Option<u32> {
          Some(self.0)
        }

        pub fn from_std(perm: Permissions) -> Self {
          use std::os::unix::fs::PermissionsExt;
          Self(perm.mode())
        }

        pub fn into_std(self) -> Option<Permissions> {
          use std::os::unix::fs::PermissionsExt;
          Some(Permissions::from_mode(self.0))
        }
      } else {
        pub fn from_mode(_mode: u32) -> Self {
          Self()
        }

        pub fn into_mode(self) -> Option<u32> {
          None
        }

        pub fn from_std(_perm: Permissions) -> Self {
          Self()
        }

        pub fn into_std(self) -> Option<Permissions> {
          None
        }
      }
  }
}
