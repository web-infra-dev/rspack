#[cfg(test)]
mod test_storage_release {
  use std::{path::PathBuf, sync::Arc};

  use rspack_fs::{MemoryFileSystem, NativeFileSystem};
  use rspack_paths::{AssertUtf8, Utf8PathBuf};
  use rspack_storage::{FileSystem, PackStorage, PackStorageOptions, Result, Storage};

  pub fn get_native_path(p: &str) -> (PathBuf, PathBuf) {
    let base = std::env::temp_dir()
      .join("rspack_test/storage/test_storage_release")
      .join(p);
    (base.join("cache"), base.join("temp"))
  }

  pub fn get_memory_path(p: &str) -> (PathBuf, PathBuf) {
    let base = PathBuf::from("/rspack_test/storage/test_storage_release/").join(p);
    (base.join("cache"), base.join("temp"))
  }

  fn create_pack_options(
    root: &Utf8PathBuf,
    temp_root: &Utf8PathBuf,
    version: &str,
    fs: Arc<FileSystem>,
  ) -> PackStorageOptions {
    PackStorageOptions {
      version: version.to_string(),
      root: root.into(),
      temp_root: temp_root.into(),
      fs,
      bucket_size: 1,
      pack_size: 1000,
      expire: 7 * 24 * 60 * 60 * 1000,
      clean: true,
      fresh_generation: Some(1),
      release_generation: Some(2),
    }
  }

  async fn get_scope_generations(storage: &PackStorage, scope_name: &str) -> Result<Vec<usize>> {
    let mut res = vec![0; 10];
    let scopes = storage.manager.scopes.lock().await;
    let scope = scopes.get(scope_name).expect("should have scope");

    for pack in scope.packs.expect_value().iter().flatten() {
      for generation in pack.generations.iter() {
        res[*generation] += 1;
      }
    }

    Ok(res)
  }

  async fn get_scope_released_count(storage: &PackStorage, scope_name: &str) -> Result<usize> {
    let mut res = 0_usize;
    let scopes = storage.manager.scopes.lock().await;
    let scope = scopes.get(scope_name).expect("should have scope");

    for pack in scope.packs.expect_value().iter().flatten() {
      if pack.contents.is_released() {
        res += pack.keys.expect_value().len();
      }
    }

    Ok(res)
  }

  async fn test_initial(
    root: &Utf8PathBuf,
    fs: Arc<FileSystem>,
    options: PackStorageOptions,
  ) -> Result<()> {
    let storage = PackStorage::new(options);
    let scope_name = "test_scope";
    let data = storage.load(scope_name).await?;
    assert!(data.is_empty());
    for i in 0..1000 {
      storage.set(
        scope_name,
        format!("key_{i:0>3}").as_bytes().to_vec(),
        format!("val_{i:0>3}").as_bytes().to_vec(),
      );
    }
    storage.trigger_save()?.await.expect("should save")?;

    assert_eq!(
      get_scope_generations(&storage, scope_name).await?,
      vec![0, 1000, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(get_scope_released_count(&storage, scope_name).await?, 0);

    for i in 0..100 {
      storage.set(
        scope_name,
        format!("key_{i:0>3}").as_bytes().to_vec(),
        format!("new_{i:0>3}").as_bytes().to_vec(),
      );
    }
    storage.trigger_save()?.await.expect("should save")?;

    assert_eq!(
      get_scope_generations(&storage, scope_name).await?,
      vec![0, 900, 100, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(get_scope_released_count(&storage, scope_name).await?, 0);

    for i in 100..200 {
      storage.set(
        scope_name,
        format!("key_{i:0>3}").as_bytes().to_vec(),
        format!("new_{i:0>3}").as_bytes().to_vec(),
      );
    }
    storage.trigger_save()?.await.expect("should save")?;

    assert_eq!(
      get_scope_generations(&storage, scope_name).await?,
      vec![0, 800, 100, 100, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(get_scope_released_count(&storage, scope_name).await?, 0);

    for i in 200..300 {
      storage.set(
        scope_name,
        format!("key_{i:0>3}").as_bytes().to_vec(),
        format!("new_{i:0>3}").as_bytes().to_vec(),
      );
    }
    storage.trigger_save()?.await.expect("should save")?;

    assert_eq!(
      get_scope_generations(&storage, scope_name).await?,
      vec![0, 700, 100, 100, 100, 0, 0, 0, 0, 0]
    );
    assert_eq!(get_scope_released_count(&storage, scope_name).await?, 660);

    for i in 300..400 {
      storage.set(
        scope_name,
        format!("key_{i:0>3}").as_bytes().to_vec(),
        format!("new_{i:0>3}").as_bytes().to_vec(),
      );
    }
    storage.trigger_save()?.await.expect("should save")?;

    assert_eq!(
      get_scope_generations(&storage, scope_name).await?,
      vec![0, 600, 100, 100, 100, 100, 0, 0, 0, 0]
    );
    assert_eq!(get_scope_released_count(&storage, scope_name).await?, 660);

    for i in 400..500 {
      storage.set(
        scope_name,
        format!("key_{i:0>3}").as_bytes().to_vec(),
        format!("new_{i:0>3}").as_bytes().to_vec(),
      );
    }
    storage.trigger_save()?.await.expect("should save")?;

    assert_eq!(
      get_scope_generations(&storage, scope_name).await?,
      vec![0, 500, 100, 100, 100, 100, 100, 0, 0, 0]
    );
    assert_eq!(get_scope_released_count(&storage, scope_name).await?, 660);

    for i in 500..600 {
      storage.set(
        scope_name,
        format!("key_{i:0>3}").as_bytes().to_vec(),
        format!("new_{i:0>3}").as_bytes().to_vec(),
      );
    }
    storage.trigger_save()?.await.expect("should save")?;

    assert!(fs.exists(&root.join("test_scope/scope_meta")).await?);
    assert_eq!(
      get_scope_generations(&storage, scope_name).await?,
      vec![0, 400, 100, 100, 100, 100, 100, 100, 0, 0]
    );
    assert_eq!(get_scope_released_count(&storage, scope_name).await?, 660);

    Ok(())
  }

  async fn test_recovery(options: PackStorageOptions) -> Result<()> {
    let storage = PackStorage::new(options);
    let data = storage.load("test_scope").await?;
    assert_eq!(data.len(), 1000);
    assert_eq!(get_scope_released_count(&storage, "test_scope").await?, 660);
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_release() -> Result<()> {
    let cases = [
      (
        get_native_path("test_release_native"),
        Arc::new(FileSystem(Arc::new(NativeFileSystem::new(false)))),
      ),
      (
        get_memory_path("test_release_memory"),
        Arc::new(FileSystem(Arc::new(MemoryFileSystem::default()))),
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

      test_initial(
        &root.join(&version),
        fs.clone(),
        create_pack_options(&root, &temp_root, &version, fs.clone()),
      )
      .await?;

      test_recovery(create_pack_options(&root, &temp_root, &version, fs.clone())).await?;
    }
    Ok(())
  }
}
