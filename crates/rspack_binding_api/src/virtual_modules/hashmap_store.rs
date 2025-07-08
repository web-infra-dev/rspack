use std::time::SystemTime;

use rspack_fs::FileMetadata;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::FxHashMap;

use super::VirtualFileStore;

#[derive(Debug, Default)]
pub struct HashMapVirtualFileStore {
  files: FxHashMap<Utf8PathBuf, (FileMetadata, Vec<u8>)>,
  directories: FxHashMap<Utf8PathBuf, FileMetadata>,
}

impl HashMapVirtualFileStore {
  pub fn new() -> Self {
    Self {
      files: FxHashMap::default(),
      directories: FxHashMap::default(),
    }
  }

  fn ensure_parent_directories(&mut self, file_path: &Utf8Path) {
    let now = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_millis() as u64;

    let dir_metadata = FileMetadata {
      is_file: false,
      is_directory: true,
      is_symlink: false,
      atime_ms: now,
      mtime_ms: now,
      ctime_ms: now,
      size: 0,
    };

    let mut current_path = file_path;

    while let Some(parent) = current_path.parent() {
      if !self.directories.contains_key(parent) {
        self.directories.insert(parent.into(), dir_metadata.clone());
        current_path = parent;
      } else {
        break;
      }
    }
  }
}

impl VirtualFileStore for HashMapVirtualFileStore {
  fn write_file(&mut self, path: &Utf8Path, content: Vec<u8>) {
    self.ensure_parent_directories(&path);

    let now = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_millis() as u64;

    let metadata = FileMetadata {
      is_file: true,
      is_directory: false,
      is_symlink: false,
      atime_ms: now,
      mtime_ms: now,
      ctime_ms: now,
      size: content.len() as u64,
    };

    self.files.insert(path.into(), (metadata, content));
  }

  fn get_file_content(&self, path: &Utf8Path) -> Option<&Vec<u8>> {
    self.files.get(path).map(|(_, content)| content)
  }

  fn get_file_metadata(&self, path: &Utf8Path) -> Option<FileMetadata> {
    self
      .files
      .get(path)
      .map(|(metadata, _)| metadata.clone())
      .or_else(|| self.directories.get(path).map(|m| m.clone()))
  }

  fn contains(&self, path: &Utf8Path) -> bool {
    self.files.contains_key(path) || self.directories.contains_key(path)
  }
}
