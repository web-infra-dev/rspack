use std::io::Read;

use rustc_hash::FxHashMap as HashMap;

use crate::db::error::{DBError, DBResult};

/// Pack file format (no magic number):
/// - Count: u32 (number of entries)
/// - Entries: [Entry; count]
///
/// Entry format:
/// - Key length: u32
/// - Key: [u8; key_len]
/// - Value length: u32
/// - Value: [u8; value_len]
#[derive(Debug)]
pub struct Pack {
  entries: Vec<(Vec<u8>, Vec<u8>)>,
}

impl Pack {
  /// Create a new Pack from entries
  pub fn new(entries: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
    Self { entries }
  }

  /// Create Pack from raw bytes
  pub fn from_bytes(data: &[u8]) -> DBResult<Self> {
    let mut cursor = std::io::Cursor::new(data);
    let mut count_bytes = [0u8; 4];
    cursor.read_exact(&mut count_bytes)?;
    let count = u32::from_le_bytes(count_bytes);

    let mut entries = Vec::with_capacity(count as usize);
    for _ in 0..count {
      let mut key_len_bytes = [0u8; 4];
      cursor.read_exact(&mut key_len_bytes)?;
      let key_len = u32::from_le_bytes(key_len_bytes) as usize;

      let mut key = vec![0u8; key_len];
      cursor.read_exact(&mut key)?;

      let mut value_len_bytes = [0u8; 4];
      cursor.read_exact(&mut value_len_bytes)?;
      let value_len = u32::from_le_bytes(value_len_bytes) as usize;

      let mut value = vec![0u8; value_len];
      cursor.read_exact(&mut value)?;

      entries.push((key, value));
    }

    Ok(Self { entries })
  }

  /// Write Pack to bytes (consumes self)
  pub fn to_bytes(self) -> DBResult<Vec<u8>> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(self.entries.len() as u32).to_le_bytes());

    for (key, value) in self.entries {
      buf.extend_from_slice(&(key.len() as u32).to_le_bytes());
      buf.extend_from_slice(&key);
      buf.extend_from_slice(&(value.len() as u32).to_le_bytes());
      buf.extend_from_slice(&value);
    }

    Ok(buf)
  }

  /// Get all entries (consumes self)
  pub fn into_entries(self) -> Vec<(Vec<u8>, Vec<u8>)> {
    self.entries
  }

  /// Convert to HashMap (consumes self)
  pub fn into_map(self) -> HashMap<Vec<u8>, Vec<u8>> {
    self.entries.into_iter().collect()
  }

  /// Merge with new entries (consumes self)
  /// New entries will override existing ones
  pub fn merge(mut self, new_entries: HashMap<Vec<u8>, Vec<u8>>) -> Self {
    let mut map: HashMap<Vec<u8>, Vec<u8>> = self.entries.into_iter().collect();
    map.extend(new_entries);
    Self {
      entries: map.into_iter().collect(),
    }
  }

  /// Get entry count
  pub fn len(&self) -> usize {
    self.entries.len()
  }
}
