use std::{
  io,
  path::{Path, PathBuf},
};

use crate::{FileMetadata, FileSystem};

fn path_to_str(path: &Path) -> &str {
  path.to_str().expect("path should be UTF-8")
}

#[derive(Default)]
pub struct MemoryFS {
  fs: vfs::MemoryFS,
}

impl MemoryFS {
  /// # Panics
  ///
  /// * Fails to create directory
  /// * Fails to write file
  #[allow(dead_code)]
  pub fn new(data: &[(&'static str, &'static str)]) -> Self {
    let mut fs = Self {
      fs: vfs::MemoryFS::default(),
    };
    for (path, content) in data {
      fs.add_file(Path::new(path), content);
    }
    fs
  }

  #[allow(dead_code)]
  pub fn add_file(&mut self, path: &Path, content: &str) {
    use vfs::FileSystem;
    let fs = &mut self.fs;
    // Create all parent directories
    for path in path.ancestors().collect::<Vec<_>>().iter().rev() {
      let path = path_to_str(path);
      if !fs.exists(path).unwrap() {
        fs.create_dir(path).unwrap();
      }
    }
    // Create file
    let mut file = fs.create_file(path_to_str(path)).unwrap();
    file.write_all(content.as_bytes()).unwrap();
  }
}
#[async_trait::async_trait]
impl FileSystem for MemoryFS {
  async fn read_to_string(&self, path: &Path) -> io::Result<String> {
    use vfs::FileSystem;
    let mut file = self
      .fs
      .open_file(path_to_str(path))
      .map_err(|err| io::Error::new(io::ErrorKind::NotFound, err))?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();
    Ok(buffer)
  }
  async fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
    let buf = self.read_to_string(path).await?;
    Ok(buf.into_bytes())
  }

  async fn metadata(&self, path: &Path) -> io::Result<FileMetadata> {
    use vfs::FileSystem;
    let metadata = self
      .fs
      .metadata(path_to_str(path))
      .map_err(|err| io::Error::new(io::ErrorKind::NotFound, err))?;
    let is_file = metadata.file_type == vfs::VfsFileType::File;
    let is_dir = metadata.file_type == vfs::VfsFileType::Directory;
    Ok(FileMetadata::new(is_file, is_dir, false))
  }

  async fn symlink_metadata(&self, path: &Path) -> io::Result<FileMetadata> {
    self.metadata(path).await
  }

  async fn canonicalize(&self, _path: &Path) -> io::Result<PathBuf> {
    Err(io::Error::new(io::ErrorKind::NotFound, "not a symlink"))
  }
}
