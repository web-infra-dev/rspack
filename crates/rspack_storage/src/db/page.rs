use rspack_paths::{Utf8Path, Utf8PathBuf};

use crate::db::{
  error::DBResult,
  index::{Index, exist_check_hash},
  options::DBOptions,
  pack::Pack,
};

/// Page represents a Hot or Cold pack file with its index
#[derive(Debug)]
pub struct Page {
  /// Page ID (e.g., "0", "1", "2")
  pub id: String,
  /// Whether this is a Hot page
  pub is_hot: bool,
  /// Path to pack file
  pub pack_path: Utf8PathBuf,
  /// Path to index file
  pub index_path: Utf8PathBuf,
}

impl Page {
  /// Create a new page
  pub fn new(base_path: &Utf8Path, id: String, is_hot: bool) -> Self {
    let suffix = if is_hot { "hot" } else { "cold" };
    let pack_path = base_path.join(format!("{}.{}.pack", id, suffix));
    let index_path = base_path.join(format!("{}.{}.index", id, suffix));

    Self {
      id,
      is_hot,
      pack_path,
      index_path,
    }
  }

  /// Write page data
  pub fn write_data<K: AsRef<[u8]>, V: AsRef<[u8]>>(
    &self,
    entries: &[(K, V)],
  ) -> DBResult<(Vec<u8>, Vec<u8>)> {
    let entries_vec: Vec<(Vec<u8>, Vec<u8>)> = entries
      .iter()
      .map(|(k, v)| (k.as_ref().to_vec(), v.as_ref().to_vec()))
      .collect();

    let pack = Pack::new(entries_vec);
    let pack_buf = pack.to_bytes()?;

    let keys: Vec<&[u8]> = entries.iter().map(|(k, _)| k.as_ref()).collect();
    let hash = exist_check_hash(&keys);
    let index = Index::new(hash);
    let index_buf = index.to_bytes();

    Ok((pack_buf, index_buf))
  }

  /// Read page data as Pack
  pub fn read_pack(&self, pack_content: &[u8]) -> DBResult<Pack> {
    Pack::from_bytes(pack_content)
  }

  /// Check if should split based on entry count
  pub fn should_split(&self, entry_count: usize, options: &DBOptions) -> bool {
    self.is_hot && entry_count > options.page_count
  }
}
