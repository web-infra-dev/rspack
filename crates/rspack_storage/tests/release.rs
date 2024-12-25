#[cfg(test)]
mod test_storage_dev {
  use std::{path::PathBuf, sync::Arc};

  use rspack_fs::{MemoryFileSystem, NativeFileSystem};
  use rspack_paths::{AssertUtf8, Utf8PathBuf};
  use rspack_storage::{
    BridgeFileSystem, FileSystem, PackStorage, PackStorageOptions, Result, Storage,
  };

  pub fn get_native_path(p: &str) -> (PathBuf, PathBuf) {
    let base = std::env::temp_dir()
      .join("rspack_test/storage/test_storage_dev")
      .join(p);
    (base.join("cache"), base.join("temp"))
  }

  pub fn get_memory_path(p: &str) -> (PathBuf, PathBuf) {
    let base = PathBuf::from("/rspack_test/storage/test_storage_dev/").join(p);
    (base.join("cache"), base.join("temp"))
  }

  fn create_pack_options(
    root: &Utf8PathBuf,
    temp_root: &Utf8PathBuf,
    version: &str,
    fs: Arc<dyn FileSystem>,
  ) -> PackStorageOptions {
    PackStorageOptions {
      version: version.to_string(),
      root: root.into(),
      temp_root: temp_root.into(),
      fs,
      bucket_size: 1,
      pack_size: 200,
      expire: 7 * 24 * 60 * 60 * 1000,
      clean: true,
      fresh_generation: Some(1),
      release_generation: Some(2),
    }
  }

  async fn test_initial_dev(
    root: &Utf8PathBuf,
    fs: Arc<dyn FileSystem>,
    options: PackStorageOptions,
  ) -> Result<()> {
    let storage = PackStorage::new(options);
    let data = storage.load("test_scope").await?;
    assert!(data.is_empty());
    for i in 0..1000 {
      storage.set(
        "test_scope",
        format!("key_{:0>3}", i).as_bytes().to_vec(),
        format!("val_{:0>3}", i).as_bytes().to_vec(),
      );
    }
    storage.trigger_save()?.await.expect("should save")?;

    for i in 0..100 {
      storage.set(
        "test_scope",
        format!("key_{:0>3}", i).as_bytes().to_vec(),
        format!("new_{:0>3}", i).as_bytes().to_vec(),
      );
    }
    storage.trigger_save()?.await.expect("should save")?;

    for i in 100..200 {
      storage.set(
        "test_scope",
        format!("key_{:0>3}", i).as_bytes().to_vec(),
        format!("new_{:0>3}", i).as_bytes().to_vec(),
      );
    }
    storage.trigger_save()?.await.expect("should save")?;

    for i in 200..300 {
      storage.set(
        "test_scope",
        format!("key_{:0>3}", i).as_bytes().to_vec(),
        format!("new_{:0>3}", i).as_bytes().to_vec(),
      );
    }
    storage.trigger_save()?.await.expect("should save")?;

    for i in 300..400 {
      storage.set(
        "test_scope",
        format!("key_{:0>3}", i).as_bytes().to_vec(),
        format!("new_{:0>3}", i).as_bytes().to_vec(),
      );
    }
    storage.trigger_save()?.await.expect("should save")?;

    for i in 400..500 {
      storage.set(
        "test_scope",
        format!("key_{:0>3}", i).as_bytes().to_vec(),
        format!("new_{:0>3}", i).as_bytes().to_vec(),
      );
    }
    storage.trigger_save()?.await.expect("should save")?;

    for i in 500..600 {
      storage.set(
        "test_scope",
        format!("key_{:0>3}", i).as_bytes().to_vec(),
        format!("new_{:0>3}", i).as_bytes().to_vec(),
      );
    }
    storage.trigger_save()?.await.expect("should save")?;

    assert!(fs.exists(&root.join("test_scope/scope_meta")).await?);
    Ok(())
  }

  async fn test_recovery_modify(options: PackStorageOptions) -> Result<()> {
    let storage = PackStorage::new(options);
    let data = storage.load("test_scope").await?;
    assert_eq!(data.len(), 1000);
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_dev() {
    let cases = [
      (
        get_native_path("test_dev_native"),
        Arc::new(BridgeFileSystem(Arc::new(NativeFileSystem {}))),
      ),
      (
        get_memory_path("test_dev_memory"),
        Arc::new(BridgeFileSystem(Arc::new(MemoryFileSystem::default()))),
      ),
    ];
    let version = "xxx".to_string();

    for ((root, temp_root), fs) in cases {
      let root = root.assert_utf8();
      let temp_root = temp_root.assert_utf8();
      fs.remove_dir(&root).await.expect("should remove root");
      fs.remove_dir(&temp_root)
        .await
        .expect("should remove temp root");

      let _ = test_initial_dev(
        &root.join(&version),
        fs.clone(),
        create_pack_options(&root, &temp_root, &version, fs.clone()),
      )
      .await
      .map_err(|e| panic!("{}", e));

      let _ = test_recovery_modify(create_pack_options(&root, &temp_root, &version, fs.clone()))
        .await
        .map_err(|e| panic!("{}", e));
    }
  }
}
