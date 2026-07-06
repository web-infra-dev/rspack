use std::fs::{Metadata, Permissions};
#[cfg(not(unix))]
use std::time::UNIX_EPOCH;

use cfg_if::cfg_if;

#[cfg(not(unix))]
use crate::IoResultToFsResultExt;
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

impl FileMetadata {
  #[cfg(unix)]
  #[inline]
  fn unix_timestamp_ms(seconds: i64, nanoseconds: i64, field: &str) -> u64 {
    let seconds = u64::try_from(seconds).unwrap_or_else(|_| panic!("{field} is before unix epoch"));
    let nanoseconds =
      u64::try_from(nanoseconds).expect("timestamp nanoseconds should not be negative");
    seconds * 1000 + nanoseconds / 1_000_000
  }

  #[cfg(unix)]
  #[inline]
  fn unix_ctime_ms(metadata: &Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;

    let ctime_ms = metadata.ctime() * 1000 + metadata.ctime_nsec() / 1_000_000;
    ctime_ms as u64
  }

  #[cfg(unix)]
  #[inline]
  fn from_metadata(metadata: Metadata) -> Self {
    use std::os::unix::fs::MetadataExt;

    let file_type = metadata.file_type();
    Self {
      is_directory: file_type.is_dir(),
      is_file: file_type.is_file(),
      is_symlink: file_type.is_symlink(),
      size: metadata.len(),
      mtime_ms: Self::unix_timestamp_ms(metadata.mtime(), metadata.mtime_nsec(), "mtime"),
      ctime_ms: Self::unix_ctime_ms(&metadata),
      atime_ms: Self::unix_timestamp_ms(metadata.atime(), metadata.atime_nsec(), "atime"),
    }
  }

  #[cfg(not(unix))]
  #[inline]
  fn system_time_ms(time: std::io::Result<std::time::SystemTime>, field: &str) -> Result<u64> {
    Ok(
      time
        .to_fs_result()?
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| panic!("{field} is before unix epoch"))
        .as_millis() as u64,
    )
  }

  #[cfg(not(unix))]
  #[inline]
  fn from_metadata(metadata: Metadata) -> Result<Self> {
    let mtime_ms = Self::system_time_ms(metadata.modified(), "mtime")?;
    let atime_ms = Self::system_time_ms(metadata.accessed(), "atime")?;

    Ok(Self {
      is_directory: metadata.is_dir(),
      is_file: metadata.is_file(),
      is_symlink: metadata.is_symlink(),
      size: metadata.len(),
      mtime_ms,
      // windows not support ctime
      ctime_ms: 0,
      atime_ms,
    })
  }
}

impl TryFrom<Metadata> for FileMetadata {
  type Error = Error;

  fn try_from(metadata: Metadata) -> Result<Self> {
    #[cfg(unix)]
    {
      Ok(Self::from_metadata(metadata))
    }
    #[cfg(not(unix))]
    {
      Self::from_metadata(metadata)
    }
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
