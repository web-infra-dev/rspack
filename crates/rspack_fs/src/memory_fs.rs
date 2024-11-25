use std::{
  collections::{HashMap, HashSet},
  sync::Mutex,
  time::{SystemTime, UNIX_EPOCH},
};

use futures::future::BoxFuture;
use rspack_paths::{AssertUtf8, Utf8Path, Utf8PathBuf};

use crate::{
  AsyncReadableFileSystem, AsyncWritableFileSystem, Error, FileMetadata, FileSystem, Result,
  SyncReadableFileSystem, SyncWritableFileSystem,
};

fn current_time() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("should get current time")
    .as_millis() as u64
}

fn new_error(msg: &str) -> Error {
  Error::Io(std::io::Error::new(std::io::ErrorKind::Other, msg))
}

#[derive(Debug)]
enum FileType {
  Dir(FileMetadata),
  File {
    content: Vec<u8>,
    metadata: FileMetadata,
  },
}

impl FileType {
  pub fn new_dir() -> FileType {
    let now = current_time();
    FileType::Dir(FileMetadata {
      is_file: false,
      is_directory: true,
      is_symlink: false,
      atime_ms: now,
      mtime_ms: now,
      ctime_ms: now,
      size: 0,
    })
  }

  pub fn new_file(content: Vec<u8>) -> FileType {
    let now = current_time();
    FileType::File {
      metadata: FileMetadata {
        is_file: true,
        is_directory: false,
        is_symlink: false,
        atime_ms: now,
        mtime_ms: now,
        ctime_ms: now,
        size: content.len() as u64,
      },
      content,
    }
  }

  pub fn metadata(&self) -> &FileMetadata {
    match self {
      Self::Dir(metadata) => metadata,
      Self::File { metadata, .. } => metadata,
    }
  }
}

#[derive(Debug, Default)]
pub struct MemoryFileSystem {
  files: Mutex<HashMap<Utf8PathBuf, FileType>>,
}
impl FileSystem for MemoryFileSystem {}

impl MemoryFileSystem {
  pub fn clear(&self) {
    let mut files = self.files.lock().expect("should get lock");
    files.clear();
  }

  fn contains_dir(&self, dir: &Utf8Path) -> Result<bool> {
    let files = self.files.lock().expect("should get lock");
    if let Some(ft) = files.get(dir) {
      if let FileType::Dir(_) = ft {
        return Ok(true);
      } else {
        return Err(new_error("invalid dir path"));
      }
    }
    Ok(false)
  }

  fn contains_file(&self, file: &Utf8Path) -> Result<bool> {
    let files = self.files.lock().expect("should get lock");
    if let Some(ft) = files.get(file) {
      if let FileType::File { .. } = ft {
        return Ok(true);
      } else {
        return Err(new_error("invalid file path"));
      }
    }
    Ok(false)
  }

  fn _remove_file(&self, file: &Utf8Path) -> Result<()> {
    if self.contains_file(file)? {
      let mut files = self.files.lock().expect("should get lock");
      files.remove(file);
    }
    Ok(())
  }

  fn _remove_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    if self.contains_dir(dir)? {
      let mut files = self.files.lock().expect("should get lock");
      files.retain(|path, _| !path.starts_with(dir));
    }
    Ok(())
  }

  fn _read_dir(&self, dir: &Utf8Path) -> Result<Vec<String>> {
    if !self.contains_dir(dir)? {
      return Err(new_error("dir not exist"));
    }

    let files = self.files.lock().expect("should get lock");
    let mut res: HashSet<String> = HashSet::default();
    for path in files.keys() {
      if let Ok(relative) = path.strip_prefix(dir) {
        if let Some(s) = relative.iter().next() {
          res.insert(s.to_string());
        }
      }
    }
    Ok(res.into_iter().collect())
  }
}

impl SyncWritableFileSystem for MemoryFileSystem {
  fn create_dir(&self, dir: &Utf8Path) -> Result<()> {
    if self.contains_dir(dir)? {
      return Ok(());
    }

    if let Some(p) = dir.parent() {
      if !self.contains_dir(p)? {
        return Err(new_error("parent directory not exist"));
      }
    }

    let mut files = self.files.lock().expect("should get lock");
    files.insert(dir.to_path_buf(), FileType::new_dir());
    Ok(())
  }

  fn create_dir_all(&self, dir: &Utf8Path) -> Result<()> {
    if self.contains_dir(dir)? {
      return Ok(());
    }

    if let Some(p) = dir.parent() {
      SyncWritableFileSystem::create_dir_all(self, p)?;
    }
    let mut files = self.files.lock().expect("should get lock");
    files.insert(dir.to_path_buf(), FileType::new_dir());
    Ok(())
  }

  fn write(&self, file: &Utf8Path, data: &[u8]) -> Result<()> {
    {
      // check file exist and update it
      let mut files = self.files.lock().expect("should get lock");
      if let Some(ft) = files.get_mut(file) {
        if let FileType::File { content, metadata } = ft {
          let now = current_time();
          *content = data.to_vec();
          metadata.mtime_ms = now;
          metadata.atime_ms = now;
          metadata.size = data.len() as u64;
          return Ok(());
        } else {
          return Err(new_error("invalid file path"));
        }
      };
    }

    // create file
    let p = file.parent().expect("should have parent dir");
    if !self.contains_dir(p)? {
      return Err(new_error("parent dir not exist"));
    }

    let mut files = self.files.lock().expect("should get lock");
    files.insert(file.to_path_buf(), FileType::new_file(data.to_vec()));
    Ok(())
  }
}

impl SyncReadableFileSystem for MemoryFileSystem {
  fn read(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    let files = self.files.lock().expect("should get lock");
    match files.get(path) {
      Some(FileType::File { content, .. }) => Ok(content.clone()),
      _ => Err(new_error("file not exist")),
    }
  }

  fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let files = self.files.lock().expect("should get lock");
    match files.get(path) {
      Some(ft) => Ok(ft.metadata().clone()),
      None => Err(new_error("file not exist")),
    }
  }

  fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    self.metadata(path)
  }

  fn canonicalize(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
    let path = dunce::canonicalize(path)?;
    Ok(path.assert_utf8())
  }
}

impl AsyncWritableFileSystem for MemoryFileSystem {
  fn create_dir<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, Result<()>> {
    let fut = async move { SyncWritableFileSystem::create_dir(self, dir) };
    Box::pin(fut)
  }

  fn create_dir_all<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, Result<()>> {
    let fut = async move { SyncWritableFileSystem::create_dir_all(self, dir) };
    Box::pin(fut)
  }

  fn write<'a>(&'a self, file: &'a Utf8Path, data: &'a [u8]) -> BoxFuture<'a, Result<()>> {
    let fut = async move { SyncWritableFileSystem::write(self, file, data) };
    Box::pin(fut)
  }

  fn remove_file<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<()>> {
    let fut = async move { self._remove_file(file) };
    Box::pin(fut)
  }

  fn remove_dir_all<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, Result<()>> {
    let fut = async move { self._remove_dir_all(dir) };
    Box::pin(fut)
  }

  fn read_dir<'a>(&'a self, dir: &'a Utf8Path) -> BoxFuture<'a, Result<Vec<String>>> {
    let fut = async move { self._read_dir(dir) };
    Box::pin(fut)
  }

  fn read_file<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<Vec<u8>>> {
    let fut = async move { SyncReadableFileSystem::read(self, file) };
    Box::pin(fut)
  }

  fn stat<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<FileMetadata>> {
    let fut = async move { SyncReadableFileSystem::metadata(self, file) };
    Box::pin(fut)
  }
}

impl AsyncReadableFileSystem for MemoryFileSystem {
  fn async_read<'a>(&'a self, file: &'a Utf8Path) -> BoxFuture<'a, Result<Vec<u8>>> {
    let fut = async move { SyncReadableFileSystem::read(self, file) };
    Box::pin(fut)
  }
}

#[cfg(test)]
mod tests {
  use rspack_paths::Utf8Path;

  use super::{
    AsyncReadableFileSystem, AsyncWritableFileSystem, MemoryFileSystem, SyncReadableFileSystem,
    SyncWritableFileSystem,
  };

  #[test]
  fn sync_fs_test() {
    let fs = MemoryFileSystem::default();
    let file_content = "1".as_bytes();
    // init fs
    SyncWritableFileSystem::create_dir_all(&fs, Utf8Path::new("/a/b/c")).unwrap();
    SyncWritableFileSystem::write(&fs, Utf8Path::new("/a/file1"), file_content).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));

    // test create_dir
    assert!(SyncWritableFileSystem::create_dir(&fs, Utf8Path::new("/a/b/c/d/e")).is_err());
    assert!(SyncWritableFileSystem::create_dir(&fs, Utf8Path::new("/a/b/c/d")).is_ok());
    assert!(SyncWritableFileSystem::create_dir(&fs, Utf8Path::new("/a/b/c/d/e")).is_ok());
    assert!(SyncWritableFileSystem::create_dir(&fs, Utf8Path::new("/a/file1/c/d")).is_err());
    assert!(SyncWritableFileSystem::create_dir(&fs, Utf8Path::new("/a/file1/c")).is_err());

    // test create_dir_all
    assert!(SyncWritableFileSystem::create_dir_all(&fs, Utf8Path::new("/a1/b1/c1")).is_ok());
    assert!(SyncWritableFileSystem::create_dir_all(&fs, Utf8Path::new("/a/file1/c/d")).is_err());
    assert!(SyncWritableFileSystem::create_dir_all(&fs, Utf8Path::new("/a/file1/c")).is_err());

    // test write
    assert!(
      SyncWritableFileSystem::write(&fs, Utf8Path::new("/a/temp/file2"), file_content).is_err()
    );
    assert!(SyncWritableFileSystem::write(&fs, Utf8Path::new("/a/file2"), file_content).is_ok());
    assert!(
      SyncWritableFileSystem::write(&fs, Utf8Path::new("/a/file1/file2"), file_content).is_err()
    );

    // read
    assert!(SyncReadableFileSystem::read(&fs, Utf8Path::new("/a/temp/file2")).is_err());
    assert!(SyncReadableFileSystem::read(&fs, Utf8Path::new("/a/file1/file2")).is_err());
    assert_eq!(
      SyncReadableFileSystem::read(&fs, Utf8Path::new("/a/file1")).unwrap(),
      file_content
    );
    assert_eq!(
      SyncReadableFileSystem::read(&fs, Utf8Path::new("/a/file2")).unwrap(),
      file_content
    );

    // metadata
    assert!(SyncReadableFileSystem::metadata(&fs, Utf8Path::new("/a/file1/c/d")).is_err());
    let file1_meta = SyncReadableFileSystem::metadata(&fs, Utf8Path::new("/a/file1")).unwrap();
    let file2_meta = SyncReadableFileSystem::metadata(&fs, Utf8Path::new("/a/file2")).unwrap();
    assert!(file1_meta.is_file);
    assert!(file2_meta.is_file);
    assert!(file1_meta.ctime_ms < file2_meta.ctime_ms);
    let dir_meta = SyncReadableFileSystem::metadata(&fs, Utf8Path::new("/a/b")).unwrap();
    assert!(dir_meta.is_directory);
    assert!(dir_meta.ctime_ms < file2_meta.ctime_ms);

    // clear
    fs.clear();
    assert!(SyncReadableFileSystem::metadata(&fs, Utf8Path::new("/a/file1")).is_err());
  }

  #[tokio::test]
  async fn async_fs_test() {
    let fs = MemoryFileSystem::default();
    let file_content = "1".as_bytes();
    // init fs
    AsyncWritableFileSystem::create_dir_all(&fs, Utf8Path::new("/a/b/c"))
      .await
      .unwrap();
    AsyncWritableFileSystem::write(&fs, Utf8Path::new("/a/file1"), file_content)
      .await
      .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // test create_dir
    assert!(
      AsyncWritableFileSystem::create_dir(&fs, Utf8Path::new("/a/b/c/d/e"))
        .await
        .is_err()
    );
    assert!(
      AsyncWritableFileSystem::create_dir(&fs, Utf8Path::new("/a/b/c/d"))
        .await
        .is_ok()
    );
    assert!(
      AsyncWritableFileSystem::create_dir(&fs, Utf8Path::new("/a/b/c/d/e"))
        .await
        .is_ok()
    );
    assert!(
      AsyncWritableFileSystem::create_dir(&fs, Utf8Path::new("/a/file1/c/d"))
        .await
        .is_err()
    );
    assert!(
      AsyncWritableFileSystem::create_dir(&fs, Utf8Path::new("/a/file1/c"))
        .await
        .is_err()
    );

    // test create_dir_all
    assert!(
      AsyncWritableFileSystem::create_dir_all(&fs, Utf8Path::new("/a1/b1/c1"))
        .await
        .is_ok()
    );
    assert!(
      AsyncWritableFileSystem::create_dir_all(&fs, Utf8Path::new("/a/file1/c/d"))
        .await
        .is_err()
    );
    assert!(
      AsyncWritableFileSystem::create_dir_all(&fs, Utf8Path::new("/a/file1/c"))
        .await
        .is_err()
    );

    // test write
    assert!(
      AsyncWritableFileSystem::write(&fs, Utf8Path::new("/a/temp/file2"), file_content)
        .await
        .is_err()
    );
    assert!(
      AsyncWritableFileSystem::write(&fs, Utf8Path::new("/a/file2"), file_content)
        .await
        .is_ok()
    );
    assert!(
      AsyncWritableFileSystem::write(&fs, Utf8Path::new("/a/file1/file2"), file_content)
        .await
        .is_err()
    );

    // read
    assert!(
      AsyncReadableFileSystem::async_read(&fs, Utf8Path::new("/a/temp/file2"))
        .await
        .is_err()
    );
    assert!(
      AsyncReadableFileSystem::async_read(&fs, Utf8Path::new("/a/file1/file2"))
        .await
        .is_err()
    );
    assert_eq!(
      AsyncReadableFileSystem::async_read(&fs, Utf8Path::new("/a/file1"))
        .await
        .unwrap(),
      file_content
    );
    assert_eq!(
      AsyncReadableFileSystem::async_read(&fs, Utf8Path::new("/a/file2"))
        .await
        .unwrap(),
      file_content
    );

    // stat
    assert!(
      AsyncWritableFileSystem::stat(&fs, Utf8Path::new("/a/file1/c/d"))
        .await
        .is_err()
    );
    let file1_meta = AsyncWritableFileSystem::stat(&fs, Utf8Path::new("/a/file1"))
      .await
      .unwrap();
    let file2_meta = AsyncWritableFileSystem::stat(&fs, Utf8Path::new("/a/file2"))
      .await
      .unwrap();
    assert!(file1_meta.is_file);
    assert!(file2_meta.is_file);
    assert!(file1_meta.ctime_ms < file2_meta.ctime_ms);
    let dir_meta = AsyncWritableFileSystem::stat(&fs, Utf8Path::new("/a/b"))
      .await
      .unwrap();
    assert!(dir_meta.is_directory);
    assert!(dir_meta.ctime_ms < file2_meta.ctime_ms);

    // read dir
    assert!(
      AsyncWritableFileSystem::read_dir(&fs, Utf8Path::new("/a2/b2/c2"))
        .await
        .is_err(),
    );
    let children = AsyncWritableFileSystem::read_dir(&fs, Utf8Path::new("/a"))
      .await
      .unwrap();
    assert_eq!(children.len(), 3);
    assert!(children.contains(&String::from("b")));
    assert!(children.contains(&String::from("file1")));
    assert!(children.contains(&String::from("file2")));

    // remove file
    assert!(
      AsyncWritableFileSystem::remove_file(&fs, Utf8Path::new("/a/b/c"))
        .await
        .is_err(),
    );
    assert!(
      AsyncWritableFileSystem::remove_file(&fs, Utf8Path::new("/a/file3"))
        .await
        .is_ok(),
    );
    assert!(
      AsyncWritableFileSystem::remove_file(&fs, Utf8Path::new("/a/file2"))
        .await
        .is_ok(),
    );
    assert!(
      AsyncWritableFileSystem::stat(&fs, Utf8Path::new("/a/file2"))
        .await
        .is_err(),
    );

    // remove dir
    assert!(
      AsyncWritableFileSystem::remove_dir_all(&fs, Utf8Path::new("/a3/b3/c3"))
        .await
        .is_ok(),
    );
    assert!(
      AsyncWritableFileSystem::remove_dir_all(&fs, Utf8Path::new("/a/file1"))
        .await
        .is_err(),
    );
    assert!(
      AsyncWritableFileSystem::remove_dir_all(&fs, Utf8Path::new("/a"))
        .await
        .is_ok(),
    );
    assert!(
      AsyncWritableFileSystem::stat(&fs, Utf8Path::new("/a/file1"))
        .await
        .is_err(),
    );
  }
}
