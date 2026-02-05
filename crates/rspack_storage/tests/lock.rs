#[cfg(test)]
mod test_storage_lock {
  use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicUsize},
  };

  use rspack_fs::{
    FileMetadata, IntermediateFileSystem, IntermediateFileSystemExtras, MemoryFileSystem,
    NativeFileSystem, ReadStream, WritableFileSystem, WriteStream,
  };
  use rspack_paths::{AssertUtf8, Utf8Path, Utf8PathBuf};
  use rspack_storage::{FileSystem, PackStorage, PackStorageOptions, Result, Storage};

  #[derive(Debug)]
  pub struct MockFileSystem {
    pub fs: Arc<dyn IntermediateFileSystem>,
    pub moved: AtomicUsize,
    pub break_on: usize,
  }

  #[async_trait::async_trait]
  impl WritableFileSystem for MockFileSystem {
    async fn create_dir(&self, dir: &Utf8Path) -> rspack_fs::Result<()> {
      self.fs.create_dir(dir).await
    }

    async fn create_dir_all(&self, dir: &Utf8Path) -> rspack_fs::Result<()> {
      self.fs.create_dir_all(dir).await
    }

    async fn write(&self, file: &Utf8Path, data: &[u8]) -> rspack_fs::Result<()> {
      self.fs.write(file, data).await
    }

    async fn remove_file(&self, file: &Utf8Path) -> rspack_fs::Result<()> {
      self.fs.remove_file(file).await
    }

    async fn remove_dir_all(&self, dir: &Utf8Path) -> rspack_fs::Result<()> {
      self.fs.remove_dir_all(dir).await
    }

    async fn read_dir(&self, dir: &Utf8Path) -> rspack_fs::Result<Vec<String>> {
      self.fs.read_dir(dir).await
    }

    async fn read_file(&self, file: &Utf8Path) -> rspack_fs::Result<Vec<u8>> {
      self.fs.read_file(file).await
    }

    async fn stat(&self, file: &Utf8Path) -> rspack_fs::Result<FileMetadata> {
      self.fs.stat(file).await
    }

    async fn set_permissions(
      &self,
      path: &Utf8Path,
      perm: rspack_fs::FilePermissions,
    ) -> rspack_fs::Result<()> {
      self.fs.set_permissions(path, perm).await
    }
  }

  #[async_trait::async_trait]
  impl IntermediateFileSystemExtras for MockFileSystem {
    async fn rename(&self, from: &Utf8Path, to: &Utf8Path) -> rspack_fs::Result<()> {
      let moved = self.moved.load(std::sync::atomic::Ordering::Relaxed);
      if moved == self.break_on {
        Err(rspack_fs::Error::Io(std::io::Error::other("move failed")))
      } else {
        self
          .moved
          .store(moved + 1, std::sync::atomic::Ordering::Relaxed);
        self.fs.rename(from, to).await
      }
    }

    async fn create_read_stream(&self, file: &Utf8Path) -> rspack_fs::Result<Box<dyn ReadStream>> {
      self.fs.create_read_stream(file).await
    }

    async fn create_write_stream(
      &self,
      file: &Utf8Path,
    ) -> rspack_fs::Result<Box<dyn WriteStream>> {
      self.fs.create_write_stream(file).await
    }
  }

  impl IntermediateFileSystem for MockFileSystem {}

  pub fn get_native_path(p: &str) -> (PathBuf, PathBuf) {
    let base = std::env::temp_dir()
      .join("rspack_test/storage/test_storage_lock")
      .join(p);
    (base.join("cache"), base.join("temp"))
  }

  pub fn get_memory_path(p: &str) -> (PathBuf, PathBuf) {
    let base = PathBuf::from("/rspack_test/storage/test_storage_lock/").join(p);
    (base.join("cache"), base.join("temp"))
  }

  async fn test_generate_lock(
    version: &str,
    root: &Utf8PathBuf,
    temp_root: &Utf8PathBuf,
    fs: Arc<FileSystem>,
  ) -> Result<()> {
    let storage = PackStorage::new(PackStorageOptions {
      version: version.to_string(),
      root: root.into(),
      temp_root: temp_root.into(),
      fs: fs.clone(),
      bucket_size: 1,
      pack_size: 100,
      expire: 7 * 24 * 60 * 60 * 1000,
      clean: true,
      fresh_generation: Some(1),
      release_generation: Some(2),
    });
    let data = storage.load("test_scope").await?;
    assert!(data.is_empty());
    for i in 0..100 {
      storage.set(
        "test_scope",
        format!("key_{i:0>3}").as_bytes().to_vec(),
        format!("val_{i:0>3}").as_bytes().to_vec(),
      );
    }
    let rx = storage.trigger_save()?;
    assert!(
      rx.await
        .expect("should save")
        .is_err_and(|e| e.to_string().contains("move failed"))
    );
    assert!(fs.exists(&root.join(version).join("move.lock")).await?);
    Ok(())
  }

  async fn test_recovery_lock(
    version: &str,
    root: &Utf8PathBuf,
    temp_root: &Utf8PathBuf,
    fs: Arc<FileSystem>,
  ) -> Result<()> {
    let storage = PackStorage::new(PackStorageOptions {
      version: version.to_string(),
      root: root.into(),
      temp_root: temp_root.into(),
      fs: fs.clone(),
      bucket_size: 1,
      pack_size: 100,
      expire: 7 * 24 * 60 * 60 * 1000,
      clean: true,
      fresh_generation: Some(1),
      release_generation: Some(2),
    });
    assert_eq!(storage.load("test_scope").await?.len(), 100);
    Ok(())
  }

  async fn test_recovery_lock_failed(
    version: &str,
    root: &Utf8PathBuf,
    temp_root: &Utf8PathBuf,
    fs: Arc<FileSystem>,
  ) -> Result<()> {
    let storage = PackStorage::new(PackStorageOptions {
      version: version.to_string(),
      root: root.into(),
      temp_root: temp_root.into(),
      fs: fs.clone(),
      bucket_size: 1,
      pack_size: 100,
      expire: 7 * 24 * 60 * 60 * 1000,
      clean: true,
      fresh_generation: Some(1),
      release_generation: Some(2),
    });
    assert!(storage.load("test_scope").await.is_err_and(|e| {
      e.to_string()
        .contains("incomplete storage due to `move.lock` from an unexpected directory")
    }));
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_consume_lock() -> Result<()> {
    let cases: [(_, Arc<dyn IntermediateFileSystem>); 2] = [
      (
        get_native_path("test_lock_native"),
        Arc::new(NativeFileSystem::new(false)),
      ),
      (
        get_memory_path("test_lock_memory"),
        Arc::new(MemoryFileSystem::default()),
      ),
    ];

    for ((root, temp_root), base_fs) in cases {
      let root = root.assert_utf8();
      let temp_root = temp_root.assert_utf8();
      let fs = Arc::new(FileSystem(base_fs.clone()));
      fs.remove_dir(&root).await.expect("should remove root");
      fs.remove_dir(&temp_root)
        .await
        .expect("should remove temp root");

      test_generate_lock(
        "xxx",
        &root,
        &temp_root,
        Arc::new(FileSystem(Arc::new(MockFileSystem {
          fs: base_fs.clone(),
          moved: AtomicUsize::new(0),
          break_on: 3,
        }))),
      )
      .await?;

      test_recovery_lock(
        "xxx",
        &root,
        &temp_root,
        Arc::new(FileSystem(Arc::new(MockFileSystem {
          fs: base_fs.clone(),
          moved: AtomicUsize::new(0),
          break_on: 9999,
        }))),
      )
      .await?;
    }
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_consume_lock_failed() -> Result<()> {
    let cases: [(_, Arc<dyn IntermediateFileSystem>); 2] = [
      (
        get_native_path("test_lock_fail_native"),
        Arc::new(NativeFileSystem::new(false)),
      ),
      (
        get_memory_path("test_lock_fail_memory"),
        Arc::new(MemoryFileSystem::default()),
      ),
    ];

    for ((root, temp_root), base_fs) in cases {
      let root = root.assert_utf8();
      let temp_root = temp_root.assert_utf8();
      let fs = Arc::new(FileSystem(base_fs.clone()));
      fs.remove_dir(&root).await.expect("should remove root");
      fs.remove_dir(&temp_root)
        .await
        .expect("should remove temp root");

      test_generate_lock(
        "xxx",
        &root,
        &temp_root,
        Arc::new(FileSystem(Arc::new(MockFileSystem {
          fs: base_fs.clone(),
          moved: AtomicUsize::new(0),
          break_on: 3,
        }))),
      )
      .await?;

      test_recovery_lock_failed(
        "xxx",
        &root,
        &temp_root.join("other"),
        Arc::new(FileSystem(Arc::new(MockFileSystem {
          fs: base_fs.clone(),
          moved: AtomicUsize::new(0),
          break_on: 9999,
        }))),
      )
      .await?;
    }
    Ok(())
  }
}
