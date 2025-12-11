use std::sync::Arc;

use rspack_core::cache::persistent::storage::{Storage, StorageOptions, create_storage};
use rspack_error::{Result, error};
use rspack_fs::{NativeFileSystem, ReadableFileSystem};
use rspack_paths::Utf8PathBuf;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::debug_info::DebugInfo;

/// Load all version storages from a directory path
/// Returns a HashMap where key is version name and value is Storage
pub fn load_storages_from_path(path: &Utf8PathBuf) -> Result<HashMap<String, Arc<dyn Storage>>> {
  let fs = Arc::new(NativeFileSystem::new(false));
  let mut storages = HashMap::default();

  // Read directory entries
  let versions = fs.read_dir_sync(path.as_path())?;

  // Collect version directories (skip hidden files)
  for v in versions {
    // Skip hidden files (starting with .)
    if v.starts_with('.') {
      continue;
    }

    // Create storage for this version
    let storage = create_storage(
      StorageOptions::FileSystem {
        directory: path.clone().into(),
      },
      v.clone(),
      fs.clone(),
    );

    storages.insert(v, storage);
  }

  Ok(storages)
}

/// Compare two sets and return error if they don't match
pub fn ensure_sets_equal<T>(
  compare_name: &str,
  iter1: impl Iterator<Item = T>,
  iter2: impl Iterator<Item = T>,
  debug_info: &DebugInfo,
) -> Result<()>
where
  T: std::fmt::Debug + std::hash::Hash + Eq,
{
  let set1: HashSet<T> = iter1.collect();
  let set2: HashSet<T> = iter2.collect();
  if set1 != set2 {
    // Find items only in set1
    let only_in_1: Vec<_> = set1.difference(&set2).collect();
    // Find items only in set2
    let only_in_2: Vec<_> = set2.difference(&set1).collect();

    let mut error_msg = format!("{} do not match:\n", compare_name);

    if !only_in_1.is_empty() {
      error_msg.push_str(&format!("  Only in path1: {:?}\n", only_in_1));
    }

    if !only_in_2.is_empty() {
      error_msg.push_str(&format!("  Only in path2: {:?}\n", only_in_2));
    }

    error_msg.push_str(&debug_info.to_string());

    return Err(error!(error_msg));
  }

  Ok(())
}
