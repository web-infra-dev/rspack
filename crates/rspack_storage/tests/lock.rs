#[cfg(test)]
mod test_storage_lock {
  use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicUsize},
  };

  use rspack_fs::{FileMetadata, MemoryFileSystem, NativeFileSystem};
  use rspack_paths::{AssertUtf8, Utf8Path, Utf8PathBuf};
  use rspack_storage::{
    BridgeFileSystem, FSError, FSOperation, FSResult, FileSystem, PackStorage, PackStorageOptions,
    Reader, Result, Storage, Writer,
  };
  use rustc_hash::FxHashSet as HashSet;

  #[derive(Debug)]
  pub(crate) struct MockFileSystem {
    pub fs: Arc<dyn FileSystem>,
    pub moved: AtomicUsize,
    pub break_on: usize,
  }

  #[async_trait::async_trait]
  impl FileSystem for MockFileSystem {
    async fn exists(&self, path: &Utf8Path) -> FSResult<bool> {
      self.fs.exists(path).await
    }

    async fn remove_dir(&self, path: &Utf8Path) -> FSResult<()> {
      self.fs.remove_dir(path).await
    }

    async fn ensure_dir(&self, path: &Utf8Path) -> FSResult<()> {
      self.fs.ensure_dir(path).await
    }

    async fn write_file(&self, path: &Utf8Path) -> FSResult<Writer> {
      self.fs.write_file(path).await
    }

    async fn read_file(&self, path: &Utf8Path) -> FSResult<Reader> {
      self.fs.read_file(path).await
    }

    async fn read_dir(&self, path: &Utf8Path) -> FSResult<HashSet<String>> {
      self.fs.read_dir(path).await
    }

    async fn metadata(&self, path: &Utf8Path) -> FSResult<FileMetadata> {
      self.fs.metadata(path).await
    }

    async fn remove_file(&self, path: &Utf8Path) -> FSResult<()> {
      self.fs.remove_file(path).await
    }

    async fn move_file(&self, from: &Utf8Path, to: &Utf8Path) -> FSResult<()> {
      let moved = self.moved.load(std::sync::atomic::Ordering::Relaxed);
      if moved == self.break_on {
        Err(FSError::from_message(
          from,
          FSOperation::Move,
          "move failed".to_string(),
        ))
      } else {
        self
          .moved
          .store(moved + 1, std::sync::atomic::Ordering::Relaxed);
        self.fs.move_file(from, to).await
      }
    }
  }

  pub(crate) fn get_native_path(p: &str) -> (PathBuf, PathBuf) {
    let base = std::env::temp_dir()
      .join("rspack_test/storage/test_storage_lock")
      .join(p);
    (base.join("cache"), base.join("temp"))
  }

  pub(crate) fn get_memory_path(p: &str) -> (PathBuf, PathBuf) {
    let base = PathBuf::from("/rspack_test/storage/test_storage_lock/").join(p);
    (base.join("cache"), base.join("temp"))
  }

  async fn test_generate_lock(
    version: &str,
    root: &Utf8PathBuf,
    temp_root: &Utf8PathBuf,
    fs: Arc<dyn FileSystem>,
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
    fs: Arc<dyn FileSystem>,
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
    fs: Arc<dyn FileSystem>,
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
    let cases = [
      (
        get_native_path("test_lock_native"),
        Arc::new(BridgeFileSystem(Arc::new(NativeFileSystem::new(false)))),
      ),
      (
        get_memory_path("test_lock_memory"),
        Arc::new(BridgeFileSystem(Arc::new(MemoryFileSystem::default()))),
      ),
    ];

    for ((root, temp_root), fs) in cases {
      let root = root.assert_utf8();
      let temp_root = temp_root.assert_utf8();
      fs.remove_dir(&root).await.expect("should remove root");
      fs.remove_dir(&temp_root)
        .await
        .expect("should remove temp root");

      test_generate_lock(
        "xxx",
        &root,
        &temp_root,
        Arc::new(MockFileSystem {
          fs: fs.clone(),
          moved: AtomicUsize::new(0),
          break_on: 3,
        }),
      )
      .await?;

      test_recovery_lock(
        "xxx",
        &root,
        &temp_root,
        Arc::new(MockFileSystem {
          fs: fs.clone(),
          moved: AtomicUsize::new(0),
          break_on: 9999,
        }),
      )
      .await?;
    }
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_consume_lock_failed() -> Result<()> {
    let cases = [
      (
        get_native_path("test_lock_fail_native"),
        Arc::new(BridgeFileSystem(Arc::new(NativeFileSystem::new(false)))),
      ),
      (
        get_memory_path("test_lock_fail_memory"),
        Arc::new(BridgeFileSystem(Arc::new(MemoryFileSystem::default()))),
      ),
    ];

    for ((root, temp_root), fs) in cases {
      let root = root.assert_utf8();
      let temp_root = temp_root.assert_utf8();
      fs.remove_dir(&root).await.expect("should remove root");
      fs.remove_dir(&temp_root)
        .await
        .expect("should remove temp root");

      test_generate_lock(
        "xxx",
        &root,
        &temp_root,
        Arc::new(MockFileSystem {
          fs: fs.clone(),
          moved: AtomicUsize::new(0),
          break_on: 3,
        }),
      )
      .await?;

      test_recovery_lock_failed(
        "xxx",
        &root,
        &temp_root.join("other"),
        Arc::new(MockFileSystem {
          fs: fs.clone(),
          moved: AtomicUsize::new(0),
          break_on: 9999,
        }),
      )
      .await?;
    }
    Ok(())
  }
}
