#[cfg(test)]
mod test_storage_multi {
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
      bucket_size: 5,
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
    let scope_data_1 = storage.load("scope_1").await?;
    let scope_data_2 = storage.load("scope_2").await?;
    assert!(scope_data_1.is_empty());
    assert!(scope_data_2.is_empty());
    for i in 0..500 {
      storage.set(
        "scope_1",
        format!("scope_1_key_{i:0>3}").as_bytes().to_vec(),
        format!("scope_1_val_{i:0>3}").as_bytes().to_vec(),
      );
      storage.set(
        "scope_2",
        format!("scope_2_key_{i:0>3}").as_bytes().to_vec(),
        format!("scope_2_val_{i:0>3}").as_bytes().to_vec(),
      );
    }
    let rx = storage.trigger_save()?;
    rx.await.expect("should save")?;
    assert!(fs.exists(&root.join("scope_1/scope_meta")).await?);
    assert!(fs.exists(&root.join("scope_2/scope_meta")).await?);
    Ok(())
  }

  async fn test_recovery_modify(
    root: &Utf8PathBuf,
    fs: Arc<dyn FileSystem>,
    options: PackStorageOptions,
  ) -> Result<()> {
    let storage = PackStorage::new(options);
    let scope_data_1 = storage.load("scope_1").await?;
    let scope_data_2 = storage.load("scope_2").await?;
    assert_eq!(scope_data_1.len(), 500);
    assert_eq!(scope_data_2.len(), 500);
    storage.set(
      "scope_1",
      format!("scope_1_key_{:0>3}", 111).as_bytes().to_vec(),
      format!("scope_1_new_{:0>3}", 111).as_bytes().to_vec(),
    );
    storage.remove(
      "scope_1",
      format!("scope_1_key_{:0>3}", 222).as_bytes().as_ref(),
    );

    storage.set(
      "scope_2",
      format!("scope_2_key_{:0>3}", 333).as_bytes().to_vec(),
      format!("scope_2_new_{:0>3}", 333).as_bytes().to_vec(),
    );
    storage.remove(
      "scope_2",
      format!("scope_2_key_{:0>3}", 444).as_bytes().as_ref(),
    );
    let rx = storage.trigger_save()?;
    rx.await.expect("should save")?;
    assert!(fs.exists(&root.join("scope_1/scope_meta")).await?);
    assert!(fs.exists(&root.join("scope_2/scope_meta")).await?);
    Ok(())
  }

  async fn test_recovery_final(
    _root: &Utf8PathBuf,
    _fs: Arc<dyn FileSystem>,
    options: PackStorageOptions,
  ) -> Result<()> {
    let storage = PackStorage::new(options);
    let scope_data_1 = storage
      .load("scope_1")
      .await?
      .into_iter()
      .map(|(k, v)| {
        (
          String::from_utf8(k.to_vec()).expect("should be utf8"),
          String::from_utf8(v.to_vec()).expect("should be utf8"),
        )
      })
      .collect::<HashMap<_, _>>();
    assert_eq!(scope_data_1.len(), 499);
    assert_eq!(
      *scope_data_1
        .get(&format!("scope_1_key_{:0>3}", 111))
        .expect("should get modified value"),
      format!("scope_1_new_{:0>3}", 111)
    );
    assert!(!scope_data_1.contains_key(&format!("scope_1_key_{:0>3}", 222)));

    let scope_data_2 = storage
      .load("scope_2")
      .await?
      .into_iter()
      .map(|(k, v)| {
        (
          String::from_utf8(k.to_vec()).expect("should be utf8"),
          String::from_utf8(v.to_vec()).expect("should be utf8"),
        )
      })
      .collect::<HashMap<_, _>>();
    assert_eq!(scope_data_2.len(), 499);
    assert_eq!(
      *scope_data_2
        .get(&format!("scope_2_key_{:0>3}", 333))
        .expect("should get modified value"),
      format!("scope_2_new_{:0>3}", 333)
    );
    assert!(!scope_data_2.contains_key(&format!("scope_2_key_{:0>3}", 444)));
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_multi() -> Result<()> {
    let cases = [
      (
        get_native_path("test_multi_native"),
        Arc::new(BridgeFileSystem(Arc::new(NativeFileSystem::new(false)))),
      ),
      (
        get_memory_path("test_multi_memory"),
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
