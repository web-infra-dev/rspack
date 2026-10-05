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
    (random_bytes(16 * 1024), 0x00),
    (b"tiny".to_vec(), 0x00),
    (Vec::new(), 0x00),
  ] {
    let fs = Arc::new(MemoryFileSystem::default());
    let mut writer = storage(&fs, 512_000);
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
      let size = u32::from_le_bytes(bytes[1..5].try_into().expect("decoded size")) as usize;
      let mut remaining = &bytes[5..];
      let mut chunks = 0;
      while !remaining.is_empty() {
        let len = u32::from_le_bytes(remaining[..4].try_into().expect("chunk length")) as usize;
        assert!(
          len > 0 && len <= remaining.len() - 4,
          "bounded chunk length"
        );
        remaining = &remaining[4 + len..];
        chunks += 1;
      }
      assert_eq!(chunks, size.div_ceil(256 * 1024));
    }
    assert_eq!(storage(&fs, 512_000).load(SCOPE).await?, vec![(key, value)]);
  }
  Ok(())
}

async fn rejects_corruption_and_rebuilds(bytes: &[u8], reason: &str) -> Result<()> {
  let fs = Arc::new(MemoryFileSystem::default());
  let mut writer = storage(&fs, 512_000);
  let value = vec![b'x'; 8 * 1024];
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

  let mut reader = storage(&fs, 512_000);
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
    storage(&fs, 512_000).load(SCOPE).await?,
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

#[tokio::test]
async fn test_multichunk_items_cross_boundaries() -> Result<()> {
  const CHUNK: usize = 256 * 1024;
  let fs = Arc::new(MemoryFileSystem::default());
  // Each item occupies 2 * CHUNK - 2 bytes including its 14-byte header.
  // Whatever the storage ordering, the second header straddles a chunk boundary,
  // and headers and values cross boundaries. Values exceed a full chunk.
  let mut expected: Vec<_> = (0..3)
    .map(|i| (vec![b'a' + i; CHUNK - 17], vec![b'x' + i; CHUNK + 1]))
    .collect();
  let mut writer = storage(&fs, 2 * 1024 * 1024);
  for (key, value) in &expected {
    writer.set(SCOPE, key.clone(), value.clone());
  }
  writer.save();
  writer.flush().await;
  drop(writer);

  let saved = packs(&fs).await?;
  let (_, bytes) = saved
    .iter()
    .max_by_key(|(_, bytes)| bytes.len())
    .expect("saved pack");
  assert_eq!(bytes[0], 0x01);
  let size = u32::from_le_bytes(bytes[1..5].try_into().expect("decoded size")) as usize;
  assert_eq!(size, 3 * (2 * CHUNK - 2));
  let mut loaded = storage(&fs, 2 * 1024 * 1024).load(SCOPE).await?;
  expected.sort_unstable();
  loaded.sort_unstable();
  assert_eq!(loaded, expected);
  Ok(())
}
