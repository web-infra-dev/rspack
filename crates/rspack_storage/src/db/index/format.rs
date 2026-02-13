use crate::db::error::{DBError, DBResult};

/// Index file format (no magic number):
/// - Hash: u64 (exist check hash)
#[derive(Debug, Clone, Copy)]
pub struct Index {
  hash: u64,
}

impl Index {
  /// Create a new Index with hash
  pub fn new(hash: u64) -> Self {
    Self { hash }
  }

  /// Create Index from bytes
  pub fn from_bytes(data: &[u8]) -> DBResult<Self> {
    if data.len() < 8 {
      return Err(DBError::InvalidFormat(
        "Index data too short, expected 8 bytes".to_string(),
      ));
    }

    let mut hash_bytes = [0u8; 8];
    hash_bytes.copy_from_slice(&data[0..8]);
    let hash = u64::from_le_bytes(hash_bytes);

    Ok(Self { hash })
  }

  /// Write Index to bytes
  pub fn to_bytes(self) -> Vec<u8> {
    self.hash.to_le_bytes().to_vec()
  }
}
