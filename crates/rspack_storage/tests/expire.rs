#[cfg(test)]
mod test_storage_expire {
  use std::{path::PathBuf, sync::Arc};

  use rspack_fs::{MemoryFileSystem, NativeFileSystem};
  use rspack_paths::{AssertUtf8, Utf8PathBuf};
  use rspack_storage::{FileSystem, PackStorage, PackStorageOptions, Result, Storage};

  pub fn get_native_path(p: &str) -> (PathBuf, PathBuf) {
    let base = std::env::temp_dir()
      .join("rspack_test/storage/test_storage_expire")
      .join(p);
    (base.join("cache"), base.join("temp"))
  }

  pub fn get_memory_path(p: &str) -> (PathBuf, PathBuf) {
    let base = PathBuf::from("/rspack_test/storage/test_storage_expire/").join(p);
    (base.join("cache"), base.join("temp"))
  }

  async fn test_initial_build(
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
      bucket_size: 2,
      pack_size: 200,
      expire: 0,
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
    rx.await.expect("should save")?;
    assert_eq!(storage.load("test_scope").await?.len(), 100);
    assert!(
      fs.exists(&root.join(version).join("test_scope/scope_meta"))
        .await?
    );
    Ok(())
  }

  async fn test_recovery_expire(
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
      bucket_size: 2,
      pack_size: 200,
      expire: 0,
      clean: true,
      fresh_generation: Some(1),
      release_generation: Some(2),
    });
    assert!(storage.load("test_scope").await.is_err_and(|e| {
      e.to_string()
        .contains("validate scope `test_scope` failed due to expiration")
    }));

    Ok(())
  }

  async fn test_remove_expired(
    last_versoin: &str,
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
      bucket_size: 2,
      pack_size: 200,
      expire: 7 * 24 * 60 * 60 * 1000,
      clean: true,
      fresh_generation: Some(1),
      release_generation: Some(2),
    });
    let data = storage.load("test_scope").await?;
    assert!(data.is_empty());
    storage.set(
      "test_scope",
      format!("key_{:0>3}", 0).as_bytes().to_vec(),
      format!("val_{:0>3}", 0).as_bytes().to_vec(),
    );
    let rx = storage.trigger_save()?;

    rx.await.expect("should save")?;
    assert!(
      fs.exists(&root.join(version).join("test_scope/scope_meta"))
        .await?
    );
    assert!(
      !(fs
        .exists(&root.join(last_versoin).join("test_scope/scope_meta"))
        .await?)
    );
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_version_expire() -> Result<()> {
    let cases = [
      (
        get_native_path("test_expire_native"),
        Arc::new(FileSystem(Arc::new(NativeFileSystem::new(false)))),
      ),
      (
        get_memory_path("test_expire_memory"),
        Arc::new(FileSystem(Arc::new(MemoryFileSystem::default()))),
      ),
    ];

    for ((root, temp_root), fs) in cases {
      let root = root.assert_utf8();
      let temp_root = temp_root.assert_utf8();
      fs.remove_dir(&root).await.expect("should remove root");
      fs.remove_dir(&temp_root)
        .await
        .expect("should remove temp root");

      test_initial_build("xxx", &root, &temp_root, fs.clone()).await?;

      test_recovery_expire("xxx", &root, &temp_root, fs.clone()).await?;

      test_remove_expired("xxx", "xxx2", &root, &temp_root, fs.clone()).await?;
    }
    Ok(())
  }
}
