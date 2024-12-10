use std::sync::Arc;

use rspack_error::Result;

mod error;
pub use error::{PackFsError, PackFsErrorOpt};
use rspack_fs::{FileMetadata, IntermediateFileSystem, ReadStream, WriteStream};
use rspack_paths::Utf8Path;
use rustc_hash::FxHashSet as HashSet;

#[async_trait::async_trait]
pub trait PackFS: std::fmt::Debug + Sync + Send {
  async fn exists(&self, path: &Utf8Path) -> Result<bool>;
  async fn remove_dir(&self, path: &Utf8Path) -> Result<()>;
  async fn ensure_dir(&self, path: &Utf8Path) -> Result<()>;
  async fn write_file(&self, path: &Utf8Path) -> Result<Box<dyn WriteStream>>;
  async fn read_file(&self, path: &Utf8Path) -> Result<Box<dyn ReadStream>>;
  async fn read_dir(&self, path: &Utf8Path) -> Result<HashSet<String>>;
  async fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata>;
  async fn remove_file(&self, path: &Utf8Path) -> Result<()>;
  async fn move_file(&self, from: &Utf8Path, to: &Utf8Path) -> Result<()>;
}

#[derive(Debug)]
pub struct PackBridgeFS(pub Arc<dyn IntermediateFileSystem>);

#[async_trait::async_trait]
impl PackFS for PackBridgeFS {
  async fn exists(&self, path: &Utf8Path) -> Result<bool> {
    match self.metadata(path).await {
      Ok(_) => Ok(true),
      Err(e) => match e.downcast::<PackFsError>() {
        Ok(e) => {
          if e.is_not_found() {
            Ok(false)
          } else {
            Err(e.into())
          }
        }
        Err(e) => Err(e),
      },
    }
  }

  async fn remove_dir(&self, path: &Utf8Path) -> Result<()> {
    if self.exists(path).await? {
      self
        .0
        .remove_dir_all(path)
        .await
        .map_err(|e| PackFsError::from_fs_error(path, PackFsErrorOpt::Remove, e))?;
    }
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

  async fn write_file(&self, path: &Utf8Path) -> Result<Box<dyn WriteStream>> {
    if self.exists(path).await? {
      self.remove_file(path).await?;
    }
    self
      .ensure_dir(path.parent().expect("should have parent"))
      .await?;

    let res = self
      .0
      .create_write_stream(path)
      .await
      .map_err(|e| PackFsError::from_fs_error(path, PackFsErrorOpt::Write, e))?;

    Ok(res)
  }

  async fn read_file(&self, path: &Utf8Path) -> Result<Box<dyn ReadStream>> {
    let res = self
      .0
      .create_read_stream(path)
      .await
      .map_err(|e| PackFsError::from_fs_error(path, PackFsErrorOpt::Read, e))?;
    Ok(res)
  }

  async fn read_dir(&self, path: &Utf8Path) -> Result<HashSet<String>> {
    let files = self
      .0
      .read_dir(path)
      .await
      .map_err(|e| PackFsError::from_fs_error(path, PackFsErrorOpt::Read, e))?;
    Ok(files.into_iter().collect::<HashSet<_>>())
  }

  async fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    let res = self
      .0
      .stat(path)
      .await
      .map_err(|e| PackFsError::from_fs_error(path, PackFsErrorOpt::Stat, e))?;
    Ok(res)
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

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_error::Result;
  use rspack_fs::MemoryFileSystem;
  use rspack_paths::Utf8PathBuf;

  use super::PackBridgeFS;
  use crate::PackFS;

  fn get_path(p: &str) -> Utf8PathBuf {
    Utf8PathBuf::from(p)
  }

  async fn test_create_dir(fs: &PackBridgeFS) -> Result<()> {
    fs.ensure_dir(&get_path("/parent/from")).await?;
    fs.ensure_dir(&get_path("/parent/to")).await?;

    assert!(fs.exists(&get_path("/parent/from")).await?);
    assert!(fs.exists(&get_path("/parent/to")).await?);

    assert!(fs.metadata(&get_path("/parent/from")).await?.is_directory);
    assert!(fs.metadata(&get_path("/parent/to")).await?.is_directory);

    Ok(())
  }

  async fn test_write_file(fs: &PackBridgeFS) -> Result<()> {
    let mut writer = fs.write_file(&get_path("/parent/from/file.txt")).await?;

    writer.write_line("hello").await?;
    writer.write(b" world").await?;
    writer.flush().await?;

    assert!(fs.exists(&get_path("/parent/from/file.txt")).await?);
    assert!(
      fs.metadata(&get_path("/parent/from/file.txt"))
        .await?
        .is_file
    );

    Ok(())
  }

  async fn test_read_file(fs: &PackBridgeFS) -> Result<()> {
    let mut reader = fs.read_file(&get_path("/parent/from/file.txt")).await?;

    assert_eq!(reader.read_line().await?, "hello");
    assert_eq!(reader.read(b" world".len()).await?, b" world");

    Ok(())
  }

  async fn test_move_file(fs: &PackBridgeFS) -> Result<()> {
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

  async fn test_remove_file(fs: &PackBridgeFS) -> Result<()> {
    fs.remove_file(&get_path("/parent/to/file.txt")).await?;
    assert!(!fs.exists(&get_path("/parent/to/file.txt")).await?);
    Ok(())
  }

  async fn test_remove_dir(fs: &PackBridgeFS) -> Result<()> {
    fs.remove_dir(&get_path("/parent/from")).await?;
    fs.remove_dir(&get_path("/parent/to")).await?;
    assert!(!fs.exists(&get_path("/parent/from")).await?);
    assert!(!fs.exists(&get_path("/parent/to")).await?);
    Ok(())
  }

  async fn test_error(fs: &PackBridgeFS) -> Result<()> {
    match fs.metadata(&get_path("/parent/from/not_exist.txt")).await {
      Ok(_) => panic!("should error"),
      Err(e) => assert_eq!(
        e.to_string(),
        r#"Rspack Storage FS Error: stat `/parent/from/not_exist.txt` failed with `Rspack FS Error: file not exist`"#
      ),
    };

    Ok(())
  }

  async fn test_memory_fs(fs: &PackBridgeFS) -> Result<()> {
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
  async fn should_pack_bridge_fs_work() {
    let fs = PackBridgeFS(Arc::new(MemoryFileSystem::default()));

    let _ = test_memory_fs(&fs).await.map_err(|e| {
      panic!("{}", e);
    });
  }
}
