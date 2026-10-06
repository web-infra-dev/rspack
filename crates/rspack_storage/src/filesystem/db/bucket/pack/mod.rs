mod generator;
mod id;
mod id_alloc;
mod load;

use std::io::ErrorKind;

use lz4_flex::block::{compress_into, compress_into_with_dict, get_maximum_output_size};

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

fn header_len(key_len: usize, value_len: usize) -> usize {
  key_len.checked_ilog10().unwrap_or(0) as usize
    + value_len.checked_ilog10().unwrap_or(0) as usize
    + 4
}

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
    // The fast encoder needs an empty dictionary or at least 4 bytes; ours is 0 or 64 KiB.
    debug_assert!(dict.is_empty() || dict.len() >= 4);
    // Errors are matched by kind in `save`. A codec error cannot happen with a
    // bound-sized scratch; if it does, the pack is stored raw.
    let written = if dict.is_empty() {
      compress_into(chunk, scratch)
    } else {
      compress_into_with_dict(chunk, scratch, dict)
    }
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
    load::load(fs, id).await
  }

  /// Saves the pack to disk and generates its index metadata.
  ///
  /// The index includes a bloom filter for fast key lookups and a content hash for integrity.
  pub async fn save(&mut self, fs: &ScopeFileSystem, id: PackId) -> Result<PackIndex> {
    // Nearby keys share the LZ4 window and make output deterministic.
    // Keys are unique within a pack, so an unstable sort is sufficient.
    self.data.sort_unstable_by(|a, b| a.0.cmp(&b.0));

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
      raw_len = raw_len
        .checked_add(header_len(key.len(), value.len()))
        .and_then(|len| len.checked_add(key.len()))
        .and_then(|len| len.checked_add(value.len()))
        .ok_or_else(|| allocation_error("serialized body size overflow".into()))?;
      index_gen.add_key(key);
      index_gen.add_value(value);
    }
    // LZ4's size prefix is u32. Larger bodies remain raw, without failing the save.
    let compressed = if raw_len >= 8 * 1024
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
        let mut key_digits = itoa::Buffer::new();
        let mut value_digits = itoa::Buffer::new();
        for (key, value) in &self.data {
          let key_len = key_digits.format(key.len()).as_bytes();
          let value_len = value_digits.format(value.len()).as_bytes();
          for mut bytes in [
            key_len,
            b" ",
            value_len,
            b"\n",
            key.as_slice(),
            value.as_slice(),
          ] {
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
        // A heap header keeps the save future small; it is held across write awaits.
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
