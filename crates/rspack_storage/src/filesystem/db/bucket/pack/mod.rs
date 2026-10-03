mod generator;
mod id;
mod id_alloc;

use std::hash::{Hash, Hasher};

use rustc_hash::FxHasher;

pub use self::{generator::PackGenerator, id::PackId, id_alloc::PackIdAlloc};
use super::{
  ScopeFileSystem,
  index::{IndexGenerator, PackIndex},
};
use crate::{Error, Result};

type PackEntries = Vec<(Vec<u8>, Vec<u8>)>;

/// File prefix so a pack is never mistaken for the previous uncompressed layout.
const PACK_MAGIC: &[u8] = b"RPK1";
const FLAG_RAW: u8 = 0;
const FLAG_LZ4: u8 = 1;
/// Packs smaller than this stay raw. Decompressing them costs more than the
/// bytes saved, and restore loads every pack even when the payload is a few
/// hundred bytes of metadata.
const MIN_COMPRESS_LEN: usize = 8 * 1024;

/// A pack file containing a collection of key-value pairs.
///
/// On-disk layout is `RPK1`, one flag byte, then the payload:
/// - flag 0: raw records
/// - flag 1: one lz4 block (prepended uncompressed length) of those records
///
/// Each record is a header line `key_len value_len`, then raw key and value
/// bytes. Items are stored in key order so repeated paths sit together in the
/// lz4 window. The fast lz4 decoder is used on purpose: the safe decoder's
/// per-byte checks dominate persistent-cache restore. The content hash still
/// covers the uncompressed keys and values in that same order.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Pack {
  data: PackEntries,
}

impl Pack {
  pub fn new(data: PackEntries) -> Self {
    Self { data }
  }

  /// Loads a pack file from disk and returns the pack data with its content hash.
  ///
  /// Returns: (Pack, content_hash)
  pub async fn load(fs: &ScopeFileSystem, id: PackId) -> Result<(Self, u64)> {
    let pack_name = id.pack_name();
    let mut reader = fs.stream_read(&pack_name).await?;
    let compressed = reader.read_to_end().await?;
    let (data, content_hash) = decode_compressed(&pack_name, &compressed)?;
    Ok((Self { data }, content_hash))
  }

  /// Saves the pack to disk and generates its index metadata.
  ///
  /// The index includes a bloom filter for fast key lookups and a content hash for integrity.
  pub async fn save(&self, fs: &ScopeFileSystem, id: PackId) -> Result<PackIndex> {
    let (compressed, index) = encode_compressed(&self.data)?;
    let mut writer = fs.stream_write(id.pack_name()).await?;
    writer.write_all(&compressed).await?;
    writer.flush().await?;
    Ok(index)
  }

  /// Consumes the pack and returns its underlying data.
  pub fn data(self) -> PackEntries {
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

fn encode_compressed(data: &[(Vec<u8>, Vec<u8>)]) -> Result<(Vec<u8>, PackIndex)> {
  let mut order: Vec<usize> = (0..data.len()).collect();
  order.sort_unstable_by(|&left, &right| data[left].0.cmp(&data[right].0));

  let mut index_gen = IndexGenerator::default();
  let mut plain = Vec::new();
  for index in order {
    let (key, value) = &data[index];
    let header = format!("{} {}\n", key.len(), value.len());
    plain.extend_from_slice(header.as_bytes());
    plain.extend_from_slice(key);
    plain.extend_from_slice(value);
    index_gen.add_key(key);
    index_gen.add_value(value);
  }
  let index = index_gen.finish();
  let mut encoded = Vec::with_capacity(PACK_MAGIC.len() + 1 + plain.len());
  encoded.extend_from_slice(PACK_MAGIC);
  if plain.len() >= MIN_COMPRESS_LEN {
    encoded.push(FLAG_LZ4);
    encoded.extend(lz4_flex::block::compress_prepend_size(&plain));
  } else {
    encoded.push(FLAG_RAW);
    encoded.extend_from_slice(&plain);
  }
  Ok((encoded, index))
}

fn decode_compressed(pack_name: &str, compressed: &[u8]) -> Result<(PackEntries, u64)> {
  let payload = pack_payload(pack_name, compressed)?;
  let plain;
  let bytes = if compressed[PACK_MAGIC.len()] == FLAG_LZ4 {
    plain = lz4_flex::block::decompress_size_prepended(payload).map_err(|error| {
      Error::InvalidFormat(format!("Failed to decompress pack '{pack_name}': {error}"))
    })?;
    plain.as_slice()
  } else {
    payload
  };
  parse_entries(pack_name, bytes)
}

fn pack_payload<'a>(pack_name: &str, compressed: &'a [u8]) -> Result<&'a [u8]> {
  let header_len = PACK_MAGIC.len() + 1;
  if compressed.len() < header_len || !compressed.starts_with(PACK_MAGIC) {
    return Err(Error::InvalidFormat(format!(
      "Invalid pack '{pack_name}': missing pack header"
    )));
  }
  match compressed[PACK_MAGIC.len()] {
    FLAG_RAW | FLAG_LZ4 => Ok(&compressed[header_len..]),
    flag => Err(Error::InvalidFormat(format!(
      "Invalid pack '{pack_name}': unknown encoding {flag}"
    ))),
  }
}

fn parse_entries(pack_name: &str, bytes: &[u8]) -> Result<(PackEntries, u64)> {
  let mut offset = 0;
  let mut data = Vec::new();
  let mut content_hasher = FxHasher::default();
  while offset < bytes.len() {
    let header_rel = bytes[offset..]
      .iter()
      .position(|byte| *byte == b'\n')
      .ok_or_else(|| {
        Error::InvalidFormat(format!(
          "Invalid pack item header in '{pack_name}': missing newline"
        ))
      })?;
    if header_rel == 0 {
      break;
    }
    let header = std::str::from_utf8(&bytes[offset..offset + header_rel]).map_err(|error| {
      Error::InvalidFormat(format!(
        "Invalid pack item header in '{pack_name}': {error}"
      ))
    })?;
    let Some((key_len, value_len)) = header.split_once(' ') else {
      return Err(Error::InvalidFormat(format!(
        "Invalid pack item header in '{pack_name}': expected 'key_len value_len', got '{header}'"
      )));
    };
    let key_len = parse_len(pack_name, "key", key_len)?;
    let value_len = parse_len(pack_name, "value", value_len)?;
    offset += header_rel + 1;
    let next = offset
      .checked_add(key_len)
      .and_then(|end| end.checked_add(value_len))
      .filter(|end| *end <= bytes.len())
      .ok_or_else(|| {
        Error::InvalidFormat(format!(
          "Invalid pack item in '{pack_name}': entry exceeds payload"
        ))
      })?;
    let key = bytes[offset..offset + key_len].to_vec();
    offset += key_len;
    let value = bytes[offset..next].to_vec();
    offset = next;
    key.hash(&mut content_hasher);
    value.hash(&mut content_hasher);
    data.push((key, value));
  }
  Ok((data, content_hasher.finish()))
}

fn parse_len(pack_name: &str, label: &str, text: &str) -> Result<usize> {
  text.parse::<usize>().map_err(|error| {
    Error::InvalidFormat(format!(
      "Failed to parse {label} length in '{pack_name}': invalid value '{text}' ({error})"
    ))
  })
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

    // A large value crosses the compression threshold. Keys stay sorted so the
    // in-memory pack matches the key order load returns.
    let compressed_id = PackId::new(11);
    let compressed = Pack::new(vec![
      ("key1".into(), vec![b'a'; 16 * 1024]),
      ("key3".into(), b"value3".to_vec()),
    ]);
    let index = compressed.save(&fs, compressed_id).await?;
    let (loaded, content_hash) = Pack::load(&fs, compressed_id).await?;
    assert!(index.check_content_hash(content_hash));
    assert_eq!(compressed, loaded);
    Ok(())
  }
}
