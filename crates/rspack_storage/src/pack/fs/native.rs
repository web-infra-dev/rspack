use std::{
  fs::{remove_dir_all, File},
  io::{BufRead, BufReader, BufWriter, Read, Write},
  os::unix::fs::MetadataExt,
};

use rspack_error::Result;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::FxHashSet as HashSet;

use super::{FileMeta, PackFileReader, PackFileWriter, PackFs, PackFsError, PackFsErrorOpt};

#[derive(Debug, Default)]
pub struct PackNativeFs;

#[async_trait::async_trait]
impl PackFs for PackNativeFs {
  async fn exists(&self, path: &Utf8Path) -> Result<bool> {
    Ok(path.exists())
  }

  async fn remove_dir(&self, path: &Utf8Path) -> Result<()> {
    if path.exists() {
      remove_dir_all(path)
        .map_err(|e| PackFsError::from_io_error(path, PackFsErrorOpt::Remove, e).into())
    } else {
      Ok(())
    }
  }

  async fn ensure_dir(&self, path: &Utf8Path) -> Result<()> {
    tokio::fs::create_dir_all(path)
      .await
      .map_err(|e| PackFsError::from_io_error(path, PackFsErrorOpt::Dir, e).into())
  }

  async fn write_file(&self, path: &Utf8Path) -> Result<Box<dyn PackFileWriter>> {
    if self.exists(path).await? {
      self.remove_file(path).await?;
    }
    self
      .ensure_dir(path.parent().expect("should have parent"))
      .await?;
    let file =
      File::create(path).map_err(|e| PackFsError::from_io_error(path, PackFsErrorOpt::Write, e))?;
    Ok(Box::new(NativeFileWriter::new(
      path.to_path_buf(),
      BufWriter::new(file),
    )))
  }

  async fn read_file(&self, path: &Utf8Path) -> Result<Box<dyn PackFileReader>> {
    let file =
      File::open(path).map_err(|e| PackFsError::from_io_error(path, PackFsErrorOpt::Read, e))?;
    Ok(Box::new(NativeFileReader::new(
      path.to_path_buf(),
      BufReader::new(file),
    )))
  }

  async fn read_dir(&self, path: &Utf8Path) -> Result<HashSet<String>> {
    let mut files = HashSet::default();
    let mut reader = tokio::fs::read_dir(path)
      .await
      .map_err(|e| PackFsError::from_io_error(path, PackFsErrorOpt::Read, e))?;

    loop {
      let entry = reader
        .next_entry()
        .await
        .map_err(|e| PackFsError::from_io_error(path, PackFsErrorOpt::Read, e))?;
      if let Some(entry) = entry {
        files.insert(entry.file_name().to_string_lossy().to_string());
      } else {
        break;
      }
    }
    Ok(files)
  }

  async fn metadata(&self, path: &Utf8Path) -> Result<FileMeta> {
    let file =
      File::open(path).map_err(|e| PackFsError::from_io_error(path, PackFsErrorOpt::Read, e))?;
    let meta_data = file
      .metadata()
      .map_err(|e| PackFsError::from_io_error(path, PackFsErrorOpt::Stat, e))?;
    Ok(FileMeta {
      is_file: meta_data.is_file(),
      is_dir: meta_data.is_dir(),
      size: meta_data.size(),
      mtime: meta_data.mtime_nsec() as u64,
    })
  }

  async fn remove_file(&self, path: &Utf8Path) -> Result<()> {
    if path.exists() {
      tokio::fs::remove_file(path)
        .await
        .map_err(|e| PackFsError::from_io_error(path, PackFsErrorOpt::Remove, e).into())
    } else {
      Ok(())
    }
  }

  async fn move_file(&self, from: &Utf8Path, to: &Utf8Path) -> Result<()> {
    if from.exists() {
      self
        .ensure_dir(&Utf8PathBuf::from(to.parent().expect("should have parent")))
        .await?;
      tokio::fs::rename(from, to)
        .await
        .map_err(|e| PackFsError::from_io_error(from, PackFsErrorOpt::Move, e).into())
    } else {
      Ok(())
    }
  }
}

#[derive(Debug)]
pub struct NativeFileWriter {
  path: Utf8PathBuf,
  inner: BufWriter<File>,
}

impl NativeFileWriter {
  pub fn new(path: Utf8PathBuf, inner: BufWriter<File>) -> Self {
    Self { path, inner }
  }
}

#[async_trait::async_trait]
impl PackFileWriter for NativeFileWriter {
  async fn line(&mut self, line: &str) -> Result<()> {
    self
      .inner
      .write_fmt(format_args!("{}\n", line))
      .map_err(|e| PackFsError::from_io_error(&self.path, PackFsErrorOpt::Write, e).into())
  }

  async fn bytes(&mut self, bytes: &[u8]) -> Result<()> {
    self
      .inner
      .write(bytes)
      .map_err(|e| PackFsError::from_io_error(&self.path, PackFsErrorOpt::Write, e))?;
    Ok(())
  }

  async fn flush(&mut self) -> Result<()> {
    Ok(())
  }

  async fn write(&mut self, content: &[u8]) -> Result<()> {
    self
      .inner
      .write_all(content)
      .map_err(|e| PackFsError::from_io_error(&self.path, PackFsErrorOpt::Write, e))?;
    Ok(())
  }
}

#[derive(Debug)]
pub struct NativeFileReader {
  path: Utf8PathBuf,
  inner: BufReader<File>,
}

impl NativeFileReader {
  pub fn new(path: Utf8PathBuf, inner: BufReader<File>) -> Self {
    Self { path, inner }
  }
}

#[async_trait::async_trait]
impl PackFileReader for NativeFileReader {
  async fn line(&mut self) -> Result<String> {
    let mut next_line = String::new();
    self
      .inner
      .read_line(&mut next_line)
      .map_err(|e| PackFsError::from_io_error(&self.path, PackFsErrorOpt::Read, e))?;

    next_line.pop();

    Ok(next_line)
  }

  async fn bytes(&mut self, len: usize) -> Result<Vec<u8>> {
    let mut bytes = vec![0u8; len];
    self
      .inner
      .read_exact(&mut bytes)
      .map_err(|e| PackFsError::from_io_error(&self.path, PackFsErrorOpt::Read, e))?;
    Ok(bytes)
  }

  async fn skip(&mut self, len: usize) -> Result<()> {
    self
      .inner
      .seek_relative(len as i64)
      .map_err(|e| PackFsError::from_io_error(&self.path, PackFsErrorOpt::Read, e).into())
  }
  async fn remain(&mut self) -> Result<Vec<u8>> {
    let mut bytes = vec![];
    self
      .inner
      .read_to_end(&mut bytes)
      .map_err(|e| PackFsError::from_io_error(&self.path, PackFsErrorOpt::Read, e))?;
    Ok(bytes)
  }
}

#[cfg(test)]
mod tests {
  use rspack_error::Result;
  use rspack_paths::{AssertUtf8, Utf8PathBuf};

  use super::PackNativeFs;
  use crate::pack::PackFs;

  fn get_path(p: &str) -> Utf8PathBuf {
    std::env::temp_dir()
      .join("./rspack_test/storage")
      .join(format!(".{p}").as_str())
      .assert_utf8()
  }

  async fn test_create_dir(fs: &PackNativeFs) -> Result<()> {
    fs.ensure_dir(&get_path("/parent/from")).await?;
    fs.ensure_dir(&get_path("/parent/to")).await?;

    assert!(fs.exists(&get_path("/parent/from")).await?);
    assert!(fs.exists(&get_path("/parent/to")).await?);

    assert!(fs.metadata(&get_path("/parent/from")).await?.is_dir);
    assert!(fs.metadata(&get_path("/parent/to")).await?.is_dir);

    Ok(())
  }

  async fn test_write_file(fs: &PackNativeFs) -> Result<()> {
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

  async fn test_read_file(fs: &PackNativeFs) -> Result<()> {
    let mut reader = fs.read_file(&get_path("/parent/from/file.txt")).await?;

    assert_eq!(reader.line().await?, "hello");
    assert_eq!(reader.bytes(b" world".len()).await?, b" world");

    Ok(())
  }

  async fn test_move_file(fs: &PackNativeFs) -> Result<()> {
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

  async fn test_remove_file(fs: &PackNativeFs) -> Result<()> {
    fs.remove_file(&get_path("/parent/to/file.txt")).await?;
    assert!(!fs.exists(&get_path("/parent/to/file.txt")).await?);
    Ok(())
  }

  async fn test_remove_dir(fs: &PackNativeFs) -> Result<()> {
    fs.remove_dir(&get_path("/parent/from")).await?;
    fs.remove_dir(&get_path("/parent/to")).await?;
    assert!(!fs.exists(&get_path("/parent/from")).await?);
    assert!(!fs.exists(&get_path("/parent/to")).await?);
    Ok(())
  }

  async fn test_error(fs: &PackNativeFs) -> Result<()> {
    match fs.metadata(&get_path("/parent/from/not_exist.txt")).await {
      Ok(_) => panic!("should error"),
      Err(e) => assert!(e
        .to_string()
        .contains("failed with `No such file or directory (os error 2)`")),
    };

    Ok(())
  }

  async fn test_native_fs(fs: &PackNativeFs) -> Result<()> {
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
  async fn should_pack_native_fs_work() {
    let fs = PackNativeFs::default();
    let _ = fs.remove_dir(&get_path("/")).await;

    let _ = test_native_fs(&fs).await.map_err(|e| {
      panic!("{}", e);
    });
  }
}
