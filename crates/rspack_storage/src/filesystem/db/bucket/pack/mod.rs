mod generator;
mod id;
mod id_alloc;

use std::{
  hash::{Hash, Hasher},
  io::ErrorKind,
};

use lz4_flex::block::{compress_into, decompress_into, get_maximum_output_size};
use rustc_hash::FxHasher;

pub use self::{generator::PackGenerator, id::PackId, id_alloc::PackIdAlloc};
use super::{
  ScopeFileSystem,
  index::{IndexGenerator, PackIndex},
};
use crate::{Error, Result};

const RAW_ENCODING: u8 = 0;
const LZ4_ENCODING: u8 = 1;

/// A pack file containing a collection of key-value pairs.
///
/// A leading encoding byte selects a raw body or an LZ4 block with a prepended
/// little-endian u32 decoded size. Small/incompressible bodies stay raw.
/// The decoded body contains `key_len value_len` headers terminated by a newline,
/// followed by key/value bytes. Content hashes cover logical keys and values,
/// not encoded file bytes.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Pack {
  data: Vec<(Vec<u8>, Vec<u8>)>,
}

impl Pack {
  pub fn new(data: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
    Self { data }
  }

  /// Loads a pack file from disk and returns the pack data with its content hash.
  ///
  /// Returns: (Pack, content_hash)
  pub async fn load(fs: &ScopeFileSystem, id: PackId) -> Result<(Self, u64)> {
    let pack_name = id.pack_name();
    let invalid =
      |reason: &str| Error::InvalidFormat(format!("Invalid pack '{pack_name}': {reason}"));
    let file_size = fs.stat(&pack_name).await?.size;
    if file_size == 0 {
      return Err(invalid("encoded size out of bounds"));
    }
    let file_size = usize::try_from(file_size)
      .map_err(|e| invalid(&format!("encoded size out of bounds: {e}")))?;
    let mut reader = fs.stream_read(&pack_name).await?;
    // Read the checked size exactly, rather than trusting a header or reading an
    // unbounded stream. An error/short read is not silently treated as EOF.
    let encoded = reader.read(file_size).await?;
    reader.close().await?;
    let (&encoding, body) = encoded
      .split_first()
      .ok_or_else(|| invalid("missing encoding"))?;
    let (body, offset) = match encoding {
      RAW_ENCODING => (encoded, 1),
      LZ4_ENCODING => {
        let size_bytes: [u8; 4] = body
          .get(..4)
          .ok_or_else(|| invalid("missing LZ4 decoded size"))?
          .try_into()
          .map_err(|_| invalid("invalid LZ4 decoded size"))?;
        let size = u32::from_le_bytes(size_bytes) as usize;
        let compressed = &body[4..];
        // LZ4 length extensions consume a byte per 255 output bytes. This
        // additional bound rejects tiny inputs claiming a huge decoded body.
        if size > compressed.len().saturating_mul(255) {
          return Err(invalid("LZ4 decoded size out of bounds"));
        }
        let mut buffer = Vec::new();
        buffer
          .try_reserve_exact(size)
          .map_err(|e| invalid(&format!("cannot allocate decoded body: {e}")))?;
        buffer.resize(size, 0);
        let written = decompress_into(compressed, &mut buffer)
          .map_err(|e| invalid(&format!("LZ4 decode failed: {e}")))?;
        if written != size {
          return Err(invalid("LZ4 decoded size mismatch"));
        }
        // Release the compressed input before allocating individual KV entries.
        drop(encoded);
        (buffer, 0)
      }
      _ => return Err(invalid("unknown encoding")),
    };

    let mut content_hasher = FxHasher::default();
    let mut data = vec![];
    let mut remaining = &body[offset..];
    while !remaining.is_empty() {
      let header_end = remaining
        .iter()
        .position(|&byte| byte == b'\n')
        .ok_or_else(|| invalid("unterminated item header"))?;
      let header = std::str::from_utf8(&remaining[..header_end])
        .map_err(|e| invalid(&format!("non-UTF8 item header: {e}")))?;
      let (key_len, value_len) = header
        .split_once(' ')
        .ok_or_else(|| invalid(&format!("expected key_len value_len, got '{header:.128}'")))?;
      let key_len = key_len
        .parse::<usize>()
        .map_err(|e| invalid(&format!("invalid key length '{key_len:.128}': {e}")))?;
      let value_len = value_len
        .parse::<usize>()
        .map_err(|e| invalid(&format!("invalid value length '{value_len:.128}': {e}")))?;
      remaining = &remaining[header_end + 1..];
      let item_len = key_len
        .checked_add(value_len)
        .filter(|&len| len <= remaining.len())
        .ok_or_else(|| invalid("item lengths exceed remaining body"))?;
      let key = remaining[..key_len].to_vec();
      let value = remaining[key_len..item_len].to_vec();
      key.hash(&mut content_hasher);
      value.hash(&mut content_hasher);
      data.push((key, value));
      remaining = &remaining[item_len..];
    }

    Ok((Self { data }, content_hasher.finish()))
  }

  /// Saves the pack to disk and generates its index metadata.
  ///
  /// The index includes a bloom filter for fast key lookups and a content hash for integrity.
  pub async fn save(&self, fs: &ScopeFileSystem, id: PackId) -> Result<PackIndex> {
    let pack_name = id.pack_name();
    // Open before encoding: stream_write yields (remove_file), and buffers built
    // before it would stay resident across that await in every concurrently
    // spawned pack save. Native writes below do not yield.
    let mut writer = fs.stream_write(&pack_name).await?;
    let allocation_error = |reason: String| {
      Error::FS(rspack_fs::Error::from(std::io::Error::new(
        ErrorKind::OutOfMemory,
        format!("Cannot allocate pack '{pack_name}': {reason}"),
      )))
    };
    let mut body = Vec::new();
    body
      .try_reserve(1)
      .map_err(|e| allocation_error(e.to_string()))?;
    // Reserve the marker from the start, so raw fallback never shifts/copies the body.
    body.push(RAW_ENCODING);
    let mut index_gen = IndexGenerator::default();
    for (key, value) in &self.data {
      let header = format!("{} {}\n", key.len(), value.len());
      let item_len = key
        .len()
        .checked_add(value.len())
        .and_then(|len| len.checked_add(body.len()))
        .and_then(|len| len.checked_add(header.len()))
        .ok_or_else(|| allocation_error("serialized body size overflow".into()))?;
      body
        .try_reserve(item_len - body.len())
        .map_err(|e| allocation_error(e.to_string()))?;
      body.extend_from_slice(header.as_bytes());
      body.extend_from_slice(key);
      body.extend_from_slice(value);
      index_gen.add_key(key);
      index_gen.add_value(value);
    }
    let body_len = body.len() - 1;
    // LZ4's size prefix is u32. Larger bodies remain raw, without failing the save.
    // The codec's output bound multiplies by 110 internally: on 32-bit targets,
    // also fall back to raw if that intermediate multiplication would overflow.
    let compressed = if body_len >= 128
      && let Ok(size) = u32::try_from(body_len)
      && body_len.checked_mul(110).is_some()
    {
      let mut bytes = Vec::new();
      let capacity = get_maximum_output_size(body_len) + 5;
      bytes
        .try_reserve_exact(capacity)
        .map_err(|e| allocation_error(e.to_string()))?;
      bytes.resize(capacity, 0);
      bytes[0] = LZ4_ENCODING;
      bytes[1..5].copy_from_slice(&size.to_le_bytes());
      let written = compress_into(&body[1..], &mut bytes[5..]).map_err(|e| {
        Error::FS(rspack_fs::Error::from(std::io::Error::other(format!(
          "Cannot encode pack '{pack_name}': {e}"
        ))))
      })?;
      bytes.truncate(written + 5);
      // Writes may yield (JS-backed FS): keep only the compressed bytes, not the
      // codec bound.
      bytes.shrink_to_fit();
      Some(bytes)
    } else {
      None
    };
    // Account for the prepended size when requiring at least 12.5% savings.
    // Both buffers already contain their marker, so neither needs a front insertion.
    let encoded = match compressed {
      Some(bytes) if bytes.len() - 1 <= body_len - body_len / 8 => {
        drop(body);
        bytes
      }
      Some(bytes) => {
        drop(bytes);
        body
      }
      None => body,
    };
    writer.write_all(&encoded).await?;
    writer.flush().await?;
    writer.close().await?;
    Ok(index_gen.finish())
  }

  /// Consumes the pack and returns its underlying data.
  pub fn data(self) -> Vec<(Vec<u8>, Vec<u8>)> {
    self.data
  }

  /// Removes all items matching the given key from the pack.
  ///
  /// Returns `true` if at least one item was removed, `false` otherwise.
  pub fn remove(&mut self, key: &[u8]) -> bool {
    let original_len = self.data.len();
    self.data.retain(|(k, _)| k.as_slice() != key);
    // Check if length changed (more efficient than comparing with original_len != current_len)
    self.data.len() < original_len
  }
}

#[cfg(test)]
mod test {
  use super::{Pack, PackId, Result, ScopeFileSystem};

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_pack() -> Result<()> {
    let pack_id = PackId::new(10);
    let fs = ScopeFileSystem::new_memory_fs("/bucket1".into());
    fs.ensure_exist().await?;

    // pack not found
    assert!(Pack::load(&fs, pack_id).await.is_err());

    let data: Vec<(Vec<u8>, Vec<u8>)> = vec![
      ("key1".into(), "value1".into()),
      ("key2".into(), "value2".into()),
      ("key3".into(), "value3".into()),
    ];
    let mut pack = Pack::new(data.clone());
    // check remove
    assert!(!pack.remove("key4".as_bytes()));
    assert!(pack.remove("key2".as_bytes()));

    let index = pack.save(&fs, pack_id).await?;
    let (other_pack, content_hash) = Pack::load(&fs, pack_id).await?;
    assert!(index.check_content_hash(content_hash));
    assert_eq!(pack, other_pack);

    // check index
    assert!(index.contains_key("key1".as_bytes()));
    // key2 has been removed
    assert!(!index.contains_key("key2".as_bytes()));
    assert!(index.contains_key("key3".as_bytes()));
    Ok(())
  }
}
