use std::{
  io::{BufRead, Cursor, Read, Seek},
  sync::Arc,
};

use rspack_error::Result;
use rspack_fs::{MemoryFileSystem, ReadableFileSystem, WritableFileSystem};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::FxHashSet as HashSet;

use super::{FileMeta, PackFileReader, PackFileWriter, PackFs, PackFsError, PackFsErrorOpt};

#[derive(Debug, Default)]
pub struct PackMemoryFs(pub Arc<MemoryFileSystem>);

#[async_trait::async_trait]
impl PackFs for PackMemoryFs {
  async fn exists(&self, path: &Utf8Path) -> Result<bool> {
    match self.0.metadata(path) {
      Ok(_) => Ok(true),
      Err(e) => {
        if e.to_string().contains("file not exist") {
          Ok(false)
        } else {
          Err(e.into())
        }
      }
    }
  }

  async fn remove_dir(&self, path: &Utf8Path) -> Result<()> {
    self
      .0
      .remove_dir_all(path)
      .await
      .map_err(|e| PackFsError::from_fs_error(path, PackFsErrorOpt::Remove, e))?;
    Ok(())
  }

  async fn ensure_dir(&self, path: &Utf8Path) -> Result<()> {
    self
      .0
      .create_dir_all(path)
      .await
      .map_err(|e| PackFsError::from_fs_error(path, PackFsErrorOpt::Dir, e))?;
    Ok(())
  }

  async fn write_file(&self, path: &Utf8Path) -> Result<Box<dyn PackFileWriter>> {
    if self.exists(path).await? {
      self.remove_file(path).await?;
    }
    self
      .ensure_dir(path.parent().expect("should have parent"))
      .await?;
    Ok(Box::new(MemoryFileWriter::new(
      path.to_path_buf(),
      self.0.clone(),
    )))
  }

  async fn read_file(&self, path: &Utf8Path) -> Result<Box<dyn PackFileReader>> {
    Ok(Box::new(MemoryFileReader::new(
      path.to_path_buf(),
      self.0.clone(),
    )))
  }

  async fn read_dir(&self, path: &Utf8Path) -> Result<HashSet<String>> {
    let files = self
      .0
      .read_dir(path)
      .await
      .map_err(|e| PackFsError::from_fs_error(path, PackFsErrorOpt::Read, e))?;
    Ok(files.into_iter().collect::<HashSet<_>>())
  }

  async fn metadata(&self, path: &Utf8Path) -> Result<FileMeta> {
    let meta_data = self
      .0
      .metadata(path)
      .map_err(|e| PackFsError::from_fs_error(path, PackFsErrorOpt::Stat, e))?;
    Ok(FileMeta {
      size: meta_data.size,
      mtime: meta_data.mtime_ms,
      is_file: meta_data.is_file,
      is_dir: meta_data.is_directory,
    })
  }

  async fn remove_file(&self, path: &Utf8Path) -> Result<()> {
    self
      .0
      .remove_file(path)
      .await
      .map_err(|e| PackFsError::from_fs_error(path, PackFsErrorOpt::Remove, e))?;
    Ok(())
  }

  async fn move_file(&self, from: &Utf8Path, to: &Utf8Path) -> Result<()> {
    if self.exists(from).await? {
      self
        .ensure_dir(to.parent().expect("should have parent"))
        .await?;
      self
        .0
        .rename(from, to)
        .await
        .map_err(|e| PackFsError::from_fs_error(from, PackFsErrorOpt::Move, e))?;
    }
    Ok(())
  }
}

#[derive(Debug)]
pub struct MemoryFileWriter {
  path: Utf8PathBuf,
  contents: Vec<u8>,
  fs: Arc<MemoryFileSystem>,
}

impl MemoryFileWriter {
  pub fn new(path: Utf8PathBuf, fs: Arc<MemoryFileSystem>) -> Self {
    Self {
      path,
      contents: vec![],
      fs,
    }
  }
}

#[async_trait::async_trait]
impl PackFileWriter for MemoryFileWriter {
  async fn line(&mut self, line: &str) -> Result<()> {
    let line = format!("{}\n", line);
    self.contents.extend(line.as_bytes().to_vec());
    Ok(())
  }

  async fn bytes(&mut self, bytes: &[u8]) -> Result<()> {
    self.contents.extend(bytes.to_vec());
    Ok(())
  }

  async fn flush(&mut self) -> Result<()> {
    self.fs.write(&self.path, &self.contents).await?;
    Ok(())
  }

  async fn write(&mut self, content: &[u8]) -> Result<()> {
    self.fs.write(&self.path, content).await?;
    Ok(())
  }
}

#[derive(Debug)]
pub struct MemoryFileReader {
  path: Utf8PathBuf,
  reader: Option<Cursor<Vec<u8>>>,
  fs: Arc<MemoryFileSystem>,
}

impl MemoryFileReader {
  pub fn new(path: Utf8PathBuf, fs: Arc<MemoryFileSystem>) -> Self {
    Self {
      path,
      reader: None,
      fs,
    }
  }
}

impl MemoryFileReader {
  async fn ensure_contents(&mut self) -> Result<()> {
    if self.reader.is_none() {
      let contents = self.fs.read_file(&self.path).await?;
      self.reader = Some(Cursor::new(contents));
    }
    Ok(())
  }
}

#[async_trait::async_trait]
impl PackFileReader for MemoryFileReader {
  async fn line(&mut self) -> Result<String> {
    self.ensure_contents().await?;

    let reader = self.reader.as_mut().expect("should have reader");
    let mut next_line = String::new();

    reader
      .read_line(&mut next_line)
      .map_err(|e| PackFsError::from_io_error(&self.path, PackFsErrorOpt::Read, e))?;

    next_line.pop();

    Ok(next_line)
  }

  async fn bytes(&mut self, len: usize) -> Result<Vec<u8>> {
    self.ensure_contents().await?;

    let mut bytes = vec![0u8; len];
    let reader = self.reader.as_mut().expect("should have reader");

    reader
      .read_exact(&mut bytes)
      .map_err(|e| PackFsError::from_io_error(&self.path, PackFsErrorOpt::Read, e))?;

    Ok(bytes)
  }

  async fn skip(&mut self, len: usize) -> Result<()> {
    self.ensure_contents().await?;

    let reader = self.reader.as_mut().expect("should have reader");

    reader
      .seek_relative(len as i64)
      .map_err(|e| PackFsError::from_io_error(&self.path, PackFsErrorOpt::Read, e).into())
  }

  async fn remain(&mut self) -> Result<Vec<u8>> {
    self.ensure_contents().await?;
    let reader = self.reader.as_mut().expect("should have reader");
    let mut bytes = vec![];
    reader
      .read_to_end(&mut bytes)
      .map_err(|e| PackFsError::from_io_error(&self.path, PackFsErrorOpt::Read, e))?;
    Ok(bytes)
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_error::Result;
  use rspack_fs::MemoryFileSystem;
  use rspack_paths::Utf8PathBuf;

  use super::PackMemoryFs;
  use crate::pack::PackFs;

  fn get_path(p: &str) -> Utf8PathBuf {
    Utf8PathBuf::from(p)
  }

  async fn test_create_dir(fs: &PackMemoryFs) -> Result<()> {
    fs.ensure_dir(&get_path("/parent/from")).await?;
    fs.ensure_dir(&get_path("/parent/to")).await?;

    assert!(fs.exists(&get_path("/parent/from")).await?);
    assert!(fs.exists(&get_path("/parent/to")).await?);

    assert!(fs.metadata(&get_path("/parent/from")).await?.is_dir);
    assert!(fs.metadata(&get_path("/parent/to")).await?.is_dir);

    Ok(())
  }

  async fn test_write_file(fs: &PackMemoryFs) -> Result<()> {
    let mut writer = fs.write_file(&get_path("/parent/from/file.txt")).await?;

    writer.line("hello").await?;
    writer.bytes(b" world").await?;
    writer.flush().await?;

    assert!(fs.exists(&get_path("/parent/from/file.txt")).await?);
    assert!(
      fs.metadata(&get_path("/parent/from/file.txt"))
        .await?
        .is_file
    );

    Ok(())
  }

  async fn test_read_file(fs: &PackMemoryFs) -> Result<()> {
    let mut reader = fs.read_file(&get_path("/parent/from/file.txt")).await?;

    assert_eq!(reader.line().await?, "hello");
    assert_eq!(reader.bytes(b" world".len()).await?, b" world");

    Ok(())
  }

  async fn test_move_file(fs: &PackMemoryFs) -> Result<()> {
    fs.move_file(
      &get_path("/parent/from/file.txt"),
      &get_path("/parent/to/file.txt"),
    )
    .await?;
    assert!(!fs.exists(&get_path("/parent/from/file.txt")).await?);
    assert!(fs.exists(&get_path("/parent/to/file.txt")).await?);
    assert!(fs.metadata(&get_path("/parent/to/file.txt")).await?.is_file);

    Ok(())
  }

  async fn test_remove_file(fs: &PackMemoryFs) -> Result<()> {
    fs.remove_file(&get_path("/parent/to/file.txt")).await?;
    assert!(!fs.exists(&get_path("/parent/to/file.txt")).await?);
    Ok(())
  }

  async fn test_remove_dir(fs: &PackMemoryFs) -> Result<()> {
    fs.remove_dir(&get_path("/parent/from")).await?;
    fs.remove_dir(&get_path("/parent/to")).await?;
    assert!(!fs.exists(&get_path("/parent/from")).await?);
    assert!(!fs.exists(&get_path("/parent/to")).await?);
    Ok(())
  }

  async fn test_error(fs: &PackMemoryFs) -> Result<()> {
    match fs.metadata(&get_path("/parent/from/not_exist.txt")).await {
      Ok(_) => panic!("should error"),
      Err(e) => assert_eq!(
        e.to_string(),
        r#"Rspack Storage FS Error: stat `/parent/from/not_exist.txt` failed with `Rspack FS Error: file not exist`"#
      ),
    };

    Ok(())
  }

  async fn test_memory_fs(fs: &PackMemoryFs) -> Result<()> {
    test_create_dir(fs).await?;
    test_write_file(fs).await?;
    test_read_file(fs).await?;
    test_move_file(fs).await?;
    test_remove_file(fs).await?;
    test_remove_dir(fs).await?;
    test_error(fs).await?;

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_pack_memory_fs_work() {
    let fs = PackMemoryFs(Arc::new(MemoryFileSystem::default()));

    let _ = test_memory_fs(&fs).await.map_err(|e| {
      panic!("{}", e);
    });
  }
}
