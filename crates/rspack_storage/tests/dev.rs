#[cfg(test)]
mod test_storage_dev {
  use std::{collections::HashMap, path::PathBuf, sync::Arc};

  use rspack_error::Result;
  use rspack_fs::{MemoryFileSystem, NativeFileSystem};
  use rspack_paths::{AssertUtf8, Utf8PathBuf};
  use rspack_storage::{PackBridgeFS, PackFS, PackStorage, PackStorageOptions, Storage};

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
    fs: Arc<dyn PackFS>,
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
    }
  }

  async fn test_initial_dev(
    root: &Utf8PathBuf,
    fs: Arc<dyn PackFS>,
    options: PackStorageOptions,
  ) -> Result<()> {
    let storage = PackStorage::new(options);
    let data = storage.load("test_scope").await?;
    assert!(data.is_empty());
    for i in 0..300 {
      storage.set(
        "test_scope",
        format!("key_{:0>3}", i).as_bytes().to_vec(),
        format!("val_{:0>3}", i).as_bytes().to_vec(),
      );
    }
    storage.trigger_save()?;

    assert_eq!(storage.load("test_scope").await?.len(), 300);
    for i in 300..700 {
      storage.set(
        "test_scope",
        format!("key_{:0>3}", i).as_bytes().to_vec(),
        format!("val_{:0>3}", i).as_bytes().to_vec(),
      );
    }
    storage.trigger_save()?;

    assert_eq!(storage.load("test_scope").await?.len(), 700);
    for i in 700..1000 {
      storage.set(
        "test_scope",
        format!("key_{:0>3}", i).as_bytes().to_vec(),
        format!("val_{:0>3}", i).as_bytes().to_vec(),
      );
    }
    let rx = storage.trigger_save()?;

    assert_eq!(storage.load("test_scope").await?.len(), 1000);
    rx.await.expect("should save")?;

    assert!(fs.exists(&root.join("test_scope/scope_meta")).await?);
    Ok(())
  }

  async fn test_recovery_modify(
    root: &Utf8PathBuf,
    fs: Arc<dyn PackFS>,
    options: PackStorageOptions,
  ) -> Result<()> {
    let storage = PackStorage::new(options);
    let data = storage.load("test_scope").await?;
    assert_eq!(data.len(), 1000);
    storage.set(
      "test_scope",
      format!("key_{:0>3}", 100).as_bytes().to_vec(),
      format!("new_{:0>3}", 100).as_bytes().to_vec(),
    );
    storage.remove("test_scope", format!("key_{:0>3}", 200).as_bytes().as_ref());
    storage.trigger_save()?;
    assert_eq!(storage.load("test_scope").await?.len(), 999);

    storage.set(
      "test_scope",
      format!("key_{:0>3}", 300).as_bytes().to_vec(),
      format!("new_{:0>3}", 300).as_bytes().to_vec(),
    );
    storage.remove("test_scope", format!("key_{:0>3}", 400).as_bytes().as_ref());
    let rx = storage.trigger_save()?;
    assert_eq!(storage.load("test_scope").await?.len(), 998);

    rx.await.expect("should save")?;
    assert!(fs.exists(&root.join("test_scope/scope_meta")).await?);
    Ok(())
  }

  async fn test_recovery_final(
    _root: &Utf8PathBuf,
    _fs: Arc<dyn PackFS>,
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
    assert_eq!(data.len(), 998);
    assert_eq!(
      *data
        .get(&format!("key_{:0>3}", 300))
        .expect("should get modified value"),
      format!("new_{:0>3}", 300)
    );
    assert!(!data.contains_key(&format!("key_{:0>3}", 400)));
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_dev() {
    let cases = [
      (
        get_native_path("test_dev_native"),
        Arc::new(PackBridgeFS(Arc::new(NativeFileSystem {}))),
      ),
      (
        get_memory_path("test_dev_memory"),
        Arc::new(PackBridgeFS(Arc::new(MemoryFileSystem::default()))),
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

      let _ = test_recovery_modify(
        &root.join(&version),
        fs.clone(),
        create_pack_options(&root, &temp_root, &version, fs.clone()),
      )
      .await
      .map_err(|e| panic!("{}", e));

      let _ = test_recovery_final(
        &root.join(&version),
        fs.clone(),
        create_pack_options(&root, &temp_root, &version, fs.clone()),
      )
      .await
      .map_err(|e| panic!("{}", e));
    }
  }
}
