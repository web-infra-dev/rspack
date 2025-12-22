mod build_dependencies;
mod occasion;
mod snapshot;

use std::{collections::VecDeque, sync::Arc};

use rspack_core::cache::persistent::storage::{Storage, StorageOptions, create_storage};
use rspack_error::{Result, error};
use rspack_fs::{NativeFileSystem, ReadableFileSystem};
use rspack_paths::Utf8PathBuf;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{debug_info::DebugInfo, utils::ensure_iter_equal};

pub fn find_relative_cache_path(root_path: &Utf8PathBuf) -> HashSet<String> {
  let fs = NativeFileSystem::new(false);
  let mut relative_paths = HashSet::default();
  let mut queue = VecDeque::new();
  queue.push_back(root_path.clone());
  loop {
    let Some(path) = queue.pop_front() else {
      break;
    };
    if matches!(path.file_name(), Some("rspack")) {
      relative_paths.insert(
        path
          .strip_prefix(root_path)
          .expect("should success")
          .to_string(),
      );
      continue;
    }

    let Ok(children) = fs.read_dir_sync(&path) else {
      continue;
    };
    for child in children {
      queue.push_back(path.join(child));
    }
  }
  relative_paths
}

/// Load all version storages from a directory path
/// Returns a HashMap where key is version name and value is Storage
pub fn load_storages_from_path(path: &Utf8PathBuf) -> HashMap<String, Arc<dyn Storage>> {
  let fs = Arc::new(NativeFileSystem::new(false));
  let mut storages = HashMap::default();

  // Read directory entries
  let Ok(versions) = fs.read_dir_sync(path.as_path()) else {
    return storages;
  };

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

  storages
}

/// Compare cache dir from two directories and return whether they are equal
pub async fn compare_cache_dir(path1: Utf8PathBuf, path2: Utf8PathBuf) -> Result<()> {
  let cache_paths1 = find_relative_cache_path(&path1);
  let cache_paths2 = find_relative_cache_path(&path2);
  let debug_info = DebugInfo::default()
    .with_field("path1", &path1.to_string())
    .with_field("path2", &path2.to_string());

  // Check if versions are identical
  ensure_iter_equal(
    "Cache directories",
    cache_paths1.iter(),
    cache_paths2.iter(),
    &debug_info,
  )?;

  for cache_relative_path in &cache_paths1 {
    let cache_path1 = path1.join(cache_relative_path);
    let cache_path2 = path2.join(cache_relative_path);
    let debug_info = DebugInfo::default()
      .with_field("path1", &cache_path1.to_string())
      .with_field("path2", &cache_path2.to_string());

    // Load storages from both paths
    let storages1 = load_storages_from_path(&cache_path1);
    let mut storages2 = load_storages_from_path(&cache_path2);

    // Check if versions are identical
    ensure_iter_equal(
      "Version directory",
      storages1.keys(),
      storages2.keys(),
      &debug_info,
    )?;

    // Compare storages for each version
    for (version, storage1) in storages1 {
      let cur_debug_info = debug_info.with_field("version", &version);

      let storage2 = storages2.remove(&version).unwrap();

      compare_storage(storage1, storage2, cur_debug_info).await?;
    }
  }

  Ok(())
}

/// Compare two storage and return whether they are equal
async fn compare_storage(
  storage1: Arc<dyn Storage>,
  storage2: Arc<dyn Storage>,
  debug_info: DebugInfo,
) -> Result<()> {
  // Get scopes from both storages
  let scopes1 = storage1.scopes().await?;
  let scopes2 = storage2.scopes().await?;

  // Check if scopes are identical
  ensure_iter_equal("Scope", scopes1.iter(), scopes2.iter(), &debug_info)?;

  // Compare each scope
  for scope in scopes1 {
    let cur_debug_info = debug_info.with_field("scope", &scope);

    match scope.as_str() {
      snapshot::SCOPE => {
        snapshot::compare(storage1.clone(), storage2.clone(), cur_debug_info).await?;
      }
      build_dependencies::SCOPE => {
        build_dependencies::compare(storage1.clone(), storage2.clone(), cur_debug_info).await?;
      }
      occasion::meta::SCOPE => {
        occasion::meta::compare(storage1.clone(), storage2.clone(), cur_debug_info).await?;
      }
      occasion::make::SCOPE => {
        occasion::make::compare(storage1.clone(), storage2.clone(), cur_debug_info).await?;
      }
      _ => {
        return Err(error!(
          "Comparison for unknown scope: {} \n{}",
          scope,
          debug_info.to_string()
        ));
      }
    }
  }

  Ok(())
}
