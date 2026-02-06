#[cfg(test)]
mod test_storage_build {
  use std::{collections::HashMap, path::PathBuf, sync::Arc};

  use rspack_fs::{MemoryFileSystem, NativeFileSystem};
  use rspack_paths::{AssertUtf8, Utf8PathBuf};
  use rspack_storage::{
    BridgeFileSystem, FileSystem, PackStorage, PackStorageOptions, Result, Storage,
  };

  pub(crate) fn get_native_path(p: &str) -> (PathBuf, PathBuf) {
    let base = std::env::temp_dir()
      .join("rspack_test/storage/test_storage_build")
      .join(p);
    (base.join("cache"), base.join("temp"))
  }

  pub(crate) fn get_memory_path(p: &str) -> (PathBuf, PathBuf) {
    let base = PathBuf::from("/rspack_test/storage/test_storage_build/").join(p);
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
      bucket_size: 10,
      pack_size: 200,
      expire: 7 * 24 * 60 * 60 * 1000,
      clean: true,
      fresh_generation: Some(1),
      release_generation: Some(2),
    }
  }

  async fn test_initial_build(
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
        format!("key_{i:0>3}").as_bytes().to_vec(),
        format!("val_{i:0>3}").as_bytes().to_vec(),
      );
    }
    let rx = storage.trigger_save()?;
    rx.await.expect("should save")?;
    assert!(fs.exists(&root.join("test_scope/scope_meta")).await?);
    Ok(())
  }

  async fn test_recovery_modify(
    root: &Utf8PathBuf,
    fs: Arc<dyn FileSystem>,
    options: PackStorageOptions,
  ) -> Result<()> {
    let storage = PackStorage::new(options);
    let data = storage.load("test_scope").await?;
    assert_eq!(data.len(), 1000);
    storage.set(
      "test_scope",
      format!("key_{:0>3}", 222).as_bytes().to_vec(),
      format!("new_{:0>3}", 222).as_bytes().to_vec(),
    );
    storage.remove("test_scope", format!("key_{:0>3}", 333).as_bytes().as_ref());
    let rx = storage.trigger_save()?;
    rx.await.expect("should save")?;
    assert!(fs.exists(&root.join("test_scope/scope_meta")).await?);
    Ok(())
  }

  async fn test_recovery_final(
    _root: &Utf8PathBuf,
    _fs: Arc<dyn FileSystem>,
    options: PackStorageOptions,
  ) -> Result<()> {
    let storage = PackStorage::new(options);
    let data = storage
      .load("test_scope")
      .await?
      .into_iter()
      .map(|(k, v)| {
        (
          String::from_utf8(k.to_vec()).expect("should be utf8"),
          String::from_utf8(v.to_vec()).expect("should be utf8"),
        )
      })
      .collect::<HashMap<_, _>>();
    assert_eq!(data.len(), 999);
    assert_eq!(
      *data
        .get(&format!("key_{:0>3}", 222))
        .expect("should get modified value"),
      format!("new_{:0>3}", 222)
    );
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_build() -> Result<()> {
    let cases = [
      (
        get_native_path("test_build_native"),
        Arc::new(BridgeFileSystem(Arc::new(NativeFileSystem::new(false)))),
      ),
      (
        get_memory_path("test_build_memory"),
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

      test_initial_build(
        &root.join(&version),
        fs.clone(),
        create_pack_options(&root, &temp_root, &version, fs.clone()),
      )
      .await?;

      test_recovery_modify(
        &root.join(&version),
        fs.clone(),
        create_pack_options(&root, &temp_root, &version, fs.clone()),
      )
      .await?;

      test_recovery_final(
        &root.join(&version),
        fs.clone(),
        create_pack_options(&root, &temp_root, &version, fs.clone()),
      )
      .await?;
    }
    Ok(())
  }
}
