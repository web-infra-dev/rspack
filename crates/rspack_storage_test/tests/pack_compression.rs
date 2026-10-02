use std::sync::Arc;

use rspack_fs::{MemoryFileSystem, WritableFileSystem};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_storage::{CacheDirectory, FileSystemOptions, FileSystemStorage, Result, Storage};

const SCOPE: &str = "values";

fn storage(fs: &Arc<MemoryFileSystem>) -> FileSystemStorage {
  FileSystemStorage::new(FileSystemOptions {
    directory: "/cache".into(),
    cache_directory: CacheDirectory::new("0123456789abcdef"),
    max_pack_size: 512_000,
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

fn random_bytes(len: usize) -> Vec<u8> {
  // Fixed seed: reproducible incompressible binary data without another dependency.
  let mut state = 0x1234_5678_u32;
  (0..len)
    .map(|_| {
      state ^= state << 13;
      state ^= state >> 17;
      state ^= state << 5;
      state.to_le_bytes()[0]
    })
    .collect()
}

#[tokio::test]
async fn test_encoding_markers_and_exact_roundtrip() -> Result<()> {
  for (value, marker) in [
    (b"synthetic repetitive text\n".repeat(1024), 0x01),
    (random_bytes(4096), 0x00),
    (b"tiny".to_vec(), 0x00),
    (Vec::new(), 0x00),
  ] {
    let fs = Arc::new(MemoryFileSystem::default());
    let mut writer = storage(&fs);
    let key = vec![0, 255, 10, 32];
    writer.set(SCOPE, key.clone(), value.clone());
    writer.save();
    writer.flush().await;
    drop(writer);

    let saved = packs(&fs).await?;
    let (_, bytes) = saved
      .iter()
      .max_by_key(|(_, bytes)| bytes.len())
      .expect("saved pack");
    assert_eq!(bytes[0], marker);
    if marker == 0x01 {
      // Frame magic, linked blocks, content size, and a 256 KiB block maximum.
      assert_eq!(&bytes[5..11], &[0x04, 0x22, 0x4d, 0x18, 0x48, 0x50]);
    }
    assert_eq!(storage(&fs).load(SCOPE).await?, vec![(key, value)]);
  }
  Ok(())
}

async fn rejects_corruption_and_rebuilds(bytes: &[u8], reason: &str) -> Result<()> {
  let fs = Arc::new(MemoryFileSystem::default());
  let mut writer = storage(&fs);
  let value = vec![b'x'; 8000];
  writer.set(SCOPE, b"key".to_vec(), value.clone());
  writer.save();
  writer.flush().await;
  drop(writer);

  let (file, _) = packs(&fs)
    .await?
    .into_iter()
    .max_by_key(|(_, bytes)| bytes.len())
    .expect("saved pack");
  fs.write(&file, bytes).await?;

  let mut reader = storage(&fs);
  let error = reader
    .load(SCOPE)
    .await
    .expect_err("corrupt pack must not load successfully");
  assert!(
    error.to_string().contains(reason),
    "expected '{reason}': {error}"
  );
  // Exercise the public miss/reset/rebuild path used by the persistent cache.
  reader.reset(SCOPE);
  reader.set(SCOPE, b"key".to_vec(), value.clone());
  reader.save();
  reader.flush().await;
  assert_eq!(
    storage(&fs).load(SCOPE).await?,
    vec![(b"key".to_vec(), value)]
  );
  Ok(())
}

#[tokio::test]
async fn test_declared_lz4_size_exceeding_ratio_is_rejected() -> Result<()> {
  // A 4 MiB decoded size exceeds 255 times this one-byte LZ4 payload.
  rejects_corruption_and_rebuilds(&[0x01, 0, 0, 64, 0, 0], "LZ4 decoded size out of bounds").await
}

#[tokio::test]
async fn test_old_format_pack_is_a_miss_and_can_be_rebuilt() -> Result<()> {
  rejects_corruption_and_rebuilds(b"3 5\nkeyvalue", "unknown encoding").await
}
