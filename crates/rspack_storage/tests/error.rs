#[cfg(test)]
mod test_storage_error {
  use std::{path::PathBuf, sync::Arc};

  use rspack_fs::{MemoryFileSystem, NativeFileSystem};
  use rspack_paths::{AssertUtf8, Utf8Path, Utf8PathBuf};
  use rspack_storage::{FileSystem, PackStorage, PackStorageOptions, Result, Storage};

  pub fn get_native_path(p: &str) -> (PathBuf, PathBuf) {
    let base = std::env::temp_dir()
      .join("rspack_test/storage/test_storage_error")
      .join(p);
    (base.join("cache"), base.join("temp"))
  }

  pub fn get_memory_path(p: &str) -> (PathBuf, PathBuf) {
    let base = PathBuf::from("/rspack_test/storage/test_storage_error/").join(p);
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
      bucket_size: 10,
      pack_size: 200,
      expire: 7 * 24 * 60 * 60 * 1000,
      clean: true,
      fresh_generation: Some(1),
      release_generation: Some(2),
    }
  }

  async fn test_initial_error(
    root: &Utf8PathBuf,
    fs: Arc<FileSystem>,
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
    assert_eq!(storage.load("test_scope").await?.len(), 1000);
    assert!(fs.exists(&root.join("test_scope/scope_meta")).await?);
    Ok(())
  }

  async fn test_recovery_invalid_meta(
    root: &Utf8PathBuf,
    fs: Arc<FileSystem>,
    options: PackStorageOptions,
  ) -> Result<()> {
    let storage = PackStorage::new(options);
    let meta_file = root.join("test_scope/scope_meta");
    let meta_content = fs.read_file(&meta_file).await?.read_to_end().await?;

    // mock
    let fake_meta_content = b"invalid meta content";
    let mut writer = fs.write_file(&meta_file).await?;
    writer.write_all(fake_meta_content).await?;
    writer.flush().await?;

    // test
    assert!(
      storage
        .load("test_scope")
        .await
        .is_err_and(|e| e.to_string().contains("parse option meta failed"))
    );

    // resume
    let mut writer = fs.write_file(&meta_file).await?;
    writer.write_all(&meta_content).await?;
    writer.flush().await?;

    Ok(())
  }

  async fn get_first_pack(
    scope_name: &str,
    meta_path: &Utf8Path,
    fs: &FileSystem,
  ) -> Result<Utf8PathBuf> {
    let mut reader = fs.read_file(meta_path).await?;
    reader.read_line().await?;
    let line = reader.read_line().await?;
    let first_pack = line
      .split(" ")
      .next()
      .expect("should have first pack")
      .split(",")
      .next()
      .expect("should have first pack");
    Ok(Utf8PathBuf::from(format!("{scope_name}/0/{first_pack}")))
  }

  async fn test_recovery_remove_pack(
    root: &Utf8PathBuf,
    fs: Arc<FileSystem>,
    options: PackStorageOptions,
  ) -> Result<()> {
    let storage = PackStorage::new(options);
    let meta_file = root.join("test_scope/scope_meta");
    let first_pack_file = root.join(&get_first_pack("test_scope", &meta_file, fs.as_ref()).await?);
    let first_pack_content = fs.read_file(&first_pack_file).await?.read_to_end().await?;

    // mock
    fs.remove_file(&first_pack_file).await?;

    // test
    assert!(storage.load("test_scope").await.is_err_and(|e| {
      e.to_string()
        .contains("validate scope `test_scope` failed due to some packs are modified")
    }));

    // resume
    let mut writer = fs.write_file(&first_pack_file).await?;
    writer.write_all(&first_pack_content).await?;
    writer.flush().await?;

    Ok(())
  }

  async fn test_recovery_modified_pack(
    root: &Utf8PathBuf,
    fs: Arc<FileSystem>,
    options: PackStorageOptions,
  ) -> Result<()> {
    let storage = PackStorage::new(options);
    let meta_file = root.join("test_scope/scope_meta");
    let first_pack_file = root.join(&get_first_pack("test_scope", &meta_file, fs.as_ref()).await?);
    let first_pack_content = fs.read_file(&first_pack_file).await?.read_to_end().await?;

    // mock
    let fake_pack_content = &first_pack_content[0..first_pack_content.len() - 1];
    let mut writer = fs.write_file(&first_pack_file).await?;
    writer.write_all(fake_pack_content).await?;
    writer.flush().await?;

    // test
    assert!(storage.load("test_scope").await.is_err_and(|e| {
      e.to_string()
        .contains("validate scope `test_scope` failed due to some packs are modified")
    }));

    // resume
    let mut writer = fs.write_file(&first_pack_file).await?;
    writer.write_all(&first_pack_content).await?;
    writer.flush().await?;

    // test
    assert!(storage.load("test_scope").await.is_ok());

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_error() -> Result<()> {
    let cases = [
      (
        get_native_path("test_error_native"),
        Arc::new(FileSystem(Arc::new(NativeFileSystem::new(false)))),
      ),
      (
        get_memory_path("test_error_memory"),
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

      test_initial_error(
        &root.join(&version),
        fs.clone(),
        create_pack_options(&root, &temp_root, &version, fs.clone()),
      )
      .await?;

      test_recovery_invalid_meta(
        &root.join(&version),
        fs.clone(),
        create_pack_options(&root, &temp_root, &version, fs.clone()),
      )
      .await?;

      test_recovery_remove_pack(
        &root.join(&version),
        fs.clone(),
        create_pack_options(&root, &temp_root, &version, fs.clone()),
      )
      .await?;

      test_recovery_modified_pack(
        &root.join(&version),
        fs.clone(),
        create_pack_options(&root, &temp_root, &version, fs.clone()),
      )
      .await?;
    }
    Ok(())
  }
}
