#[cfg(test)]
mod test_storage_lock {
  use std::{
    path::PathBuf,
    sync::{atomic::AtomicUsize, Arc},
  };

  use rspack_error::{error, Result};
  use rspack_fs::{FileMetadata, MemoryFileSystem, NativeFileSystem, ReadStream, WriteStream};
  use rspack_paths::{AssertUtf8, Utf8Path, Utf8PathBuf};
  use rspack_storage::{PackBridgeFS, PackFS, PackStorage, PackStorageOptions, Storage};
  use rustc_hash::FxHashSet as HashSet;

  #[derive(Debug)]
  pub struct MockPackFS {
    pub fs: Arc<dyn PackFS>,
    pub moved: AtomicUsize,
    pub break_on: usize,
  }

  #[async_trait::async_trait]
  impl PackFS for MockPackFS {
    async fn exists(&self, path: &Utf8Path) -> Result<bool> {
      self.fs.exists(path).await
    }

    async fn remove_dir(&self, path: &Utf8Path) -> Result<()> {
      self.fs.remove_dir(path).await
    }

    async fn ensure_dir(&self, path: &Utf8Path) -> Result<()> {
      self.fs.ensure_dir(path).await
    }

    async fn write_file(&self, path: &Utf8Path) -> Result<Box<dyn WriteStream>> {
      self.fs.write_file(path).await
    }

    async fn read_file(&self, path: &Utf8Path) -> Result<Box<dyn ReadStream>> {
      self.fs.read_file(path).await
    }

    async fn read_dir(&self, path: &Utf8Path) -> Result<HashSet<String>> {
      self.fs.read_dir(path).await
    }

    async fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
      self.fs.metadata(path).await
    }

    async fn remove_file(&self, path: &Utf8Path) -> Result<()> {
      self.fs.remove_file(path).await
    }

    async fn move_file(&self, from: &Utf8Path, to: &Utf8Path) -> Result<()> {
      let moved = self.moved.load(std::sync::atomic::Ordering::Relaxed);
      if moved == self.break_on {
        Err(error!("move failed"))
      } else {
        self
          .moved
          .store(moved + 1, std::sync::atomic::Ordering::Relaxed);
        self.fs.move_file(from, to).await
      }
    }
  }

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
    fs: Arc<dyn PackFS>,
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
    });
    let data = storage.load("test_scope").await?;
    assert!(data.is_empty());
    for i in 0..100 {
      storage.set(
        "test_scope",
        format!("key_{:0>3}", i).as_bytes().to_vec(),
        format!("val_{:0>3}", i).as_bytes().to_vec(),
      );
    }
    let rx = storage.trigger_save()?;
    assert_eq!(storage.load("test_scope").await?.len(), 100);

    assert!(rx
      .await
      .expect("should save")
      .is_err_and(|e| e.to_string().contains("move failed")));
    assert!(fs.exists(&root.join(version).join("move.lock")).await?);
    Ok(())
  }

  async fn test_recovery_lock(
    version: &str,
    root: &Utf8PathBuf,
    temp_root: &Utf8PathBuf,
    fs: Arc<dyn PackFS>,
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
    });
    assert_eq!(storage.load("test_scope").await?.len(), 100);
    Ok(())
  }

  async fn test_recovery_lock_failed(
    version: &str,
    root: &Utf8PathBuf,
    temp_root: &Utf8PathBuf,
    fs: Arc<dyn PackFS>,
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
    });
    assert!(storage.load("test_scope").await.is_err_and(|e| {
      e.to_string()
        .contains("incomplete storage due to `move.lock` from an unexpected directory")
    }));
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_consume_lock() {
    let cases = [
      (
        get_native_path("test_lock_native"),
        Arc::new(PackBridgeFS(Arc::new(NativeFileSystem {}))),
      ),
      (
        get_memory_path("test_lock_memory"),
        Arc::new(PackBridgeFS(Arc::new(MemoryFileSystem::default()))),
      ),
    ];

    for ((root, temp_root), fs) in cases {
      let root = root.assert_utf8();
      let temp_root = temp_root.assert_utf8();
      fs.remove_dir(&root).await.expect("should remove root");
      fs.remove_dir(&temp_root)
        .await
        .expect("should remove temp root");

      let _ = test_generate_lock(
        "xxx",
        &root,
        &temp_root,
        Arc::new(MockPackFS {
          fs: fs.clone(),
          moved: AtomicUsize::new(0),
          break_on: 3,
        }),
      )
      .await
      .map_err(|e| panic!("{}", e));

      let _ = test_recovery_lock(
        "xxx",
        &root,
        &temp_root,
        Arc::new(MockPackFS {
          fs: fs.clone(),
          moved: AtomicUsize::new(0),
          break_on: 9999,
        }),
      )
      .await
      .map_err(|e| panic!("{}", e));
    }
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_consume_lock_failed() {
    let cases = [
      (
        get_native_path("test_lock_fail_native"),
        Arc::new(PackBridgeFS(Arc::new(NativeFileSystem {}))),
      ),
      (
        get_memory_path("test_lock_fail_memory"),
        Arc::new(PackBridgeFS(Arc::new(MemoryFileSystem::default()))),
      ),
    ];

    for ((root, temp_root), fs) in cases {
      let root = root.assert_utf8();
      let temp_root = temp_root.assert_utf8();
      fs.remove_dir(&root).await.expect("should remove root");
      fs.remove_dir(&temp_root)
        .await
        .expect("should remove temp root");

      let _ = test_generate_lock(
        "xxx",
        &root,
        &temp_root,
        Arc::new(MockPackFS {
          fs: fs.clone(),
          moved: AtomicUsize::new(0),
          break_on: 3,
        }),
      )
      .await
      .map_err(|e| panic!("{}", e));

      let _ = test_recovery_lock_failed(
        "xxx",
        &root,
        &temp_root.join("other"),
        Arc::new(MockPackFS {
          fs: fs.clone(),
          moved: AtomicUsize::new(0),
          break_on: 9999,
        }),
      )
      .await
      .map_err(|e| panic!("{}", e));
    }
  }
}
