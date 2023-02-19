use std::{fs, path::Path};

use super::{
  sync::{ReadableFileSystem, WritableFileSystem},
  Error, Result,
};

pub struct NativeFileSystem;

impl WritableFileSystem for NativeFileSystem {
  fn create_dir<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
    fs::create_dir(dir.as_ref()).map_err(Error::from)
  }

  fn create_dir_all<P: AsRef<std::path::Path>>(&self, dir: P) -> Result<()> {
    fs::create_dir_all(dir.as_ref()).map_err(Error::from)
  }

  fn write<P: AsRef<std::path::Path>, D: AsRef<[u8]>>(
    &self,
    file: P,
    data: D,
  ) -> crate::Result<()> {
    fs::write(file.as_ref(), data.as_ref()).map_err(Error::from)
  }
}

impl ReadableFileSystem for NativeFileSystem {
  fn read<P: AsRef<Path>>(&self, file: P) -> Result<Vec<u8>> {
    fs::read(file.as_ref()).map_err(Error::from)
  }
}
