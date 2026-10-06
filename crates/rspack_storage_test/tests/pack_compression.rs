use std::sync::Arc;

use rspack_fs::{MemoryFileSystem, WritableFileSystem};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_storage::{CacheDirectory, FileSystemOptions, FileSystemStorage, Result, Storage};

const SCOPE: &str = "values";

fn storage(fs: &Arc<MemoryFileSystem>, max_pack_size: usize) -> FileSystemStorage {
  FileSystemStorage::new(FileSystemOptions {
    directory: "/cache".into(),
    cache_directory: CacheDirectory::new("0123456789abcdef"),
    max_pack_size,
    compression: true,
    expire: 0,
    fs: fs.clone(),
  })
}

fn bucket_path() -> Utf8PathBuf {
  Utf8Path::new("/cache")
    .join(CacheDirectory::new("0123456789abcdef").as_str())
    .join(SCOPE)
}

async fn packs(fs: &MemoryFileSystem) -> Result<Vec<(Utf8PathBuf, Vec<u8>)>> {
  let bucket = bucket_path();
  let mut result = Vec::new();
  for name in WritableFileSystem::read_dir(fs, &bucket).await? {
    if name.ends_with(".pack") {
      let file = bucket.join(name);
      let bytes = fs.read_file(&file).await?;
      result.push((file, bytes));
    }
  }
  Ok(result)
}

#[tokio::test]
async fn test_declared_lz4_size_exceeding_ratio_is_rejected() -> Result<()> {
  let fs = Arc::new(MemoryFileSystem::default());
  let mut writer = storage(&fs, 512_000);
  let value = vec![b'x'; 8 * 1024];
  writer.set(SCOPE, b"key".to_vec(), value);
  writer.save();
  writer.flush().await;
  drop(writer);

  let (file, _) = packs(&fs)
    .await?
    .into_iter()
    .max_by_key(|(_, bytes)| bytes.len())
    .expect("saved pack");
  // A 4 MiB decoded size exceeds 255 times this one-byte LZ4 payload.
  fs.write(&file, &[0x01, 0, 0, 64, 0, 0]).await?;

  let reader = storage(&fs, 512_000);
  let error = reader
    .load(SCOPE)
    .await
    .expect_err("corrupt pack must not load successfully");
  assert!(
    error.to_string().contains("LZ4 decoded size out of bounds"),
    "expected 'LZ4 decoded size out of bounds': {error}"
  );
  Ok(())
}
