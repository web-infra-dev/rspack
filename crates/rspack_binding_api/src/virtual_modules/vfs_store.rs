use rspack_fs::{Error, FileMetadata, MemoryFileSystem, ReadableFileSystem, Result};
use rspack_paths::Utf8Path;

use crate::virtual_modules::VirtualFileStore;

pub struct VfsVirtualFileStore {
  fs: MemoryFileSystem,
}

impl VfsVirtualFileStore {
  pub fn new() -> Self {
    Self {
      fs: MemoryFileSystem::default(),
    }
  }
}

impl VirtualFileStore for VfsVirtualFileStore {
  fn write_file(&mut self, path: &Utf8Path, content: Vec<u8>) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
      Error::new(
        std::io::ErrorKind::InvalidInput,
        "virtual file must have a parent directory",
      )
    })?;
    self.fs.create_dir_all_sync(parent)?;
    self.fs.write_sync(path, &content)
  }

  fn get_file_content(&self, path: &Utf8Path) -> Option<Vec<u8>> {
    self.fs.read_sync(path).ok()
  }

  fn get_file_metadata(&self, path: &Utf8Path) -> Option<FileMetadata> {
    self.fs.metadata_sync(path).ok()
  }

  fn contains(&self, path: &Utf8Path) -> bool {
    self.fs.metadata_sync(path).is_ok()
  }

  fn read_dir(&self, path: &Utf8Path) -> Option<Vec<String>> {
    self.fs.read_dir_sync(path).ok()
  }
}
