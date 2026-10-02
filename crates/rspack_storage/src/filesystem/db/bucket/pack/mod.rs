mod generator;
mod id;
mod id_alloc;

use std::{
  hash::{Hash, Hasher},
  io::ErrorKind,
};

use lz4_flex::block::{
  compress_into_with_dict, decompress_into_with_dict, get_maximum_output_size,
};
use rustc_hash::FxHasher;

pub use self::{generator::PackGenerator, id::PackId, id_alloc::PackIdAlloc};
use super::{
  ScopeFileSystem,
  index::{IndexGenerator, PackIndex},
};
use crate::{Error, Result};

const RAW_ENCODING: u8 = 0;
const LZ4_ENCODING: u8 = 1;
const CHUNK: usize = 256 * 1024;
const DICT: usize = 64 * 1024;

struct CappedOutput {
  bytes: Vec<u8>,
  cap: usize,
}

impl CappedOutput {
  fn compress_chunk(
    &mut self,
    staging: &mut Vec<u8>,
    dict_len: &mut usize,
    scratch: &mut [u8],
  ) -> std::io::Result<()> {
    let (dict, chunk) = staging.split_at(*dict_len);
    // Errors are matched by kind in `save`. A codec error cannot happen with a
    // bound-sized scratch; if it does, the pack is stored raw.
    let written = compress_into_with_dict(chunk, scratch, dict)
      .map_err(|_| std::io::Error::from(ErrorKind::FileTooLarge))?;
    if 4 + written > self.cap - self.bytes.len() {
      return Err(ErrorKind::FileTooLarge.into());
    }
    self
      .bytes
      .try_reserve(4 + written)
      .map_err(|_| std::io::Error::from(ErrorKind::OutOfMemory))?;
    self
      .bytes
      .extend_from_slice(&(written as u32).to_le_bytes());
    self.bytes.extend_from_slice(&scratch[..written]);
    let staged_len = staging.len();
    *dict_len = DICT.min(staged_len);
    staging.copy_within(staged_len - *dict_len.., 0);
    staging.truncate(*dict_len);
    Ok(())
  }
}

/// A pack file containing a collection of key-value pairs.
///
/// Marker 0x00 selects a raw body. Marker 0x01 selects a little-endian u32
/// decoded size followed by chunks, each a little-endian u32 compressed length
/// and an LZ4 block. Each chunk decodes to min(256 KiB, remaining decoded size),
/// so only the last chunk can be shorter. Blocks use the preceding up-to-64 KiB
/// raw bytes as their dictionary. No trailing bytes are allowed.
/// Small/incompressible bodies stay raw.
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
        let mut remaining = compressed;
        let mut pos = 0;
        while pos < size {
          let len_bytes: [u8; 4] = remaining
            .get(..4)
            .ok_or_else(|| invalid("missing LZ4 chunk length"))?
            .try_into()
            .map_err(|_| invalid("invalid LZ4 chunk length"))?;
          let len = u32::from_le_bytes(len_bytes) as usize;
          remaining = &remaining[4..];
          let block = remaining
            .get(..len)
            .ok_or_else(|| invalid("LZ4 chunk length out of bounds"))?;
          let n = CHUNK.min(size - pos);
          // Zero-fill each chunk just before decoding into it (capacity is reserved).
          buffer.resize(pos + n, 0);
          let (head, tail) = buffer.split_at_mut(pos);
          let dict = &head[pos.saturating_sub(DICT)..];
          let written = decompress_into_with_dict(block, &mut tail[..n], dict)
            .map_err(|e| invalid(&format!("LZ4 decode failed: {e}")))?;
          if written != n {
            return Err(invalid("LZ4 decoded chunk size mismatch"));
          }
          remaining = &remaining[len..];
          pos += n;
        }
        if !remaining.is_empty() {
          return Err(invalid("trailing bytes after LZ4 chunks"));
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
    // Use append writes: not every filesystem's write_all appends to the stream.
    async fn write_bytes(writer: &mut dyn rspack_fs::WriteStream, mut bytes: &[u8]) -> Result<()> {
      while !bytes.is_empty() {
        let written = writer.write(bytes).await?;
        if written == 0 {
          return Err(Error::FS(rspack_fs::Error::from(std::io::Error::from(
            ErrorKind::WriteZero,
          ))));
        }
        bytes = &bytes[written..];
      }
      Ok(())
    }

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
    let mut raw_len = 0usize;
    let mut index_gen = IndexGenerator::default();
    for (key, value) in &self.data {
      let header = format!("{} {}\n", key.len(), value.len());
      raw_len = raw_len
        .checked_add(header.len())
        .and_then(|len| len.checked_add(key.len()))
        .and_then(|len| len.checked_add(value.len()))
        .ok_or_else(|| allocation_error("serialized body size overflow".into()))?;
      index_gen.add_key(key);
      index_gen.add_value(value);
    }
    // LZ4's size prefix is u32. Larger bodies remain raw, without failing the save.
    let compressed = if raw_len >= 128
      && let Ok(size) = u32::try_from(raw_len)
    {
      // Account for the prepended size when requiring at least 12.5% savings.
      let mut sink = CappedOutput {
        bytes: Vec::new(),
        cap: raw_len - raw_len / 8 - 4,
      };
      // Size buffers for the pack: chunks never exceed `first`. After the first
      // chunk, staging holds at most DICT + min(CHUNK, rest), which fits in
      // min(DICT, rest) + first because DICT <= CHUNK.
      let first = CHUNK.min(raw_len);
      let rest = raw_len - first;
      let mut staging = Vec::new();
      staging
        .try_reserve_exact(DICT.min(rest) + first)
        .map_err(|e| allocation_error(e.to_string()))?;
      let mut scratch = Vec::new();
      let scratch_len = get_maximum_output_size(first);
      scratch
        .try_reserve_exact(scratch_len)
        .map_err(|e| allocation_error(e.to_string()))?;
      scratch.resize(scratch_len, 0);
      let encoded = (|| -> std::io::Result<CappedOutput> {
        let mut dict_len = 0;
        for (key, value) in &self.data {
          let header = format!("{} {}\n", key.len(), value.len());
          for mut bytes in [header.as_bytes(), key.as_slice(), value.as_slice()] {
            while !bytes.is_empty() {
              let n = bytes.len().min(dict_len + CHUNK - staging.len());
              staging.extend_from_slice(&bytes[..n]);
              bytes = &bytes[n..];
              if staging.len() == dict_len + CHUNK {
                sink.compress_chunk(&mut staging, &mut dict_len, &mut scratch)?;
              }
            }
          }
        }
        if staging.len() > dict_len {
          sink.compress_chunk(&mut staging, &mut dict_len, &mut scratch)?;
        }
        Ok(sink)
      })();
      match encoded {
        Ok(sink) => Some((size, sink.bytes)),
        Err(e) if e.kind() == ErrorKind::FileTooLarge => None,
        Err(e) if e.kind() == ErrorKind::OutOfMemory => {
          return Err(allocation_error(e.to_string()));
        }
        Err(e) => {
          return Err(Error::FS(rspack_fs::Error::from(std::io::Error::other(
            format!("Cannot encode pack '{pack_name}': {e}"),
          ))));
        }
      }
    } else {
      None
    };
    if let Some((size, bytes)) = compressed {
      let [a, b, c, d] = size.to_le_bytes();
      write_bytes(writer.as_mut(), &[LZ4_ENCODING, a, b, c, d]).await?;
      write_bytes(writer.as_mut(), &bytes).await?;
    } else {
      write_bytes(writer.as_mut(), &[RAW_ENCODING]).await?;
      for (key, value) in &self.data {
        let header = format!("{} {}\n", key.len(), value.len());
        write_bytes(writer.as_mut(), header.as_bytes()).await?;
        write_bytes(writer.as_mut(), key).await?;
        write_bytes(writer.as_mut(), value).await?;
      }
    }
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
