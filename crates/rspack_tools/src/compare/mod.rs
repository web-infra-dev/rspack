mod occasion;
mod snapshot;

use std::{collections::VecDeque, sync::Arc};

use rspack_core::cache::persistent::storage::{BoxStorage, StorageOptions, create_storage};
use rspack_error::{Result, error};
use rspack_fs::{NativeFileSystem, ReadableFileSystem};
use rspack_paths::Utf8PathBuf;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{debug_info::DebugInfo, utils::ensure_iter_equal};

const META_FILE_NAME: &str = "_meta";

fn is_cache_scope(name: &str) -> bool {
  name == occasion::meta::SCOPE || name == occasion::make::SCOPE
}

fn has_version_scope(fs: &NativeFileSystem, path: &Utf8PathBuf) -> bool {
  let Ok(versions) = fs.read_dir_sync(path.as_path()) else {
    return false;
  };

  versions
    .into_iter()
    .filter(|version| !version.starts_with(['.', '_']))
    .map(|version| path.join(version))
    .any(|version_path| {
      fs.metadata_sync(&version_path)
        .is_ok_and(|metadata| metadata.is_directory)
        && fs
          .read_dir_sync(&version_path)
          .is_ok_and(|scopes| scopes.iter().any(|scope| is_cache_scope(scope)))
    })
}

fn is_storage_root(fs: &NativeFileSystem, path: &Utf8PathBuf) -> bool {
  fs.metadata_sync(&path.join(META_FILE_NAME))
    .is_ok_and(|metadata| metadata.is_file)
    || has_version_scope(fs, path)
}

pub fn find_relative_cache_path(root_path: &Utf8PathBuf) -> HashSet<String> {
  let fs = NativeFileSystem::new(false);
  let mut relative_paths = HashSet::default();
  let mut queue = VecDeque::new();
  queue.push_back(root_path.clone());
  while let Some(path) = queue.pop_front() {
    if is_storage_root(&fs, &path) {
      relative_paths.insert(
        path
          .strip_prefix(root_path)
          .expect("should succeed")
          .to_string(),
      );
      continue;
    }

    let Ok(children) = fs.read_dir_sync(&path) else {
      continue;
    };
    for child in children {
      if child.starts_with(['.', '_']) {
        continue;
      }
      let child_path = path.join(child);
      if fs
        .metadata_sync(&child_path)
        .is_ok_and(|metadata| metadata.is_directory)
      {
        queue.push_back(child_path);
      }
    }
  }
  relative_paths
}

fn join_relative_cache_path(root_path: &Utf8PathBuf, relative_path: &str) -> Utf8PathBuf {
  if relative_path.is_empty() {
    root_path.clone()
  } else {
    root_path.join(relative_path)
  }
}

/// Load all version storages from a directory path.
/// Returns a HashMap where key is `<version>` and value is BoxStorage.
pub fn load_storages_from_path(path: &Utf8PathBuf) -> HashMap<String, BoxStorage> {
  let fs = Arc::new(NativeFileSystem::new(false));
  let mut storages = HashMap::default();

  let Ok(versions) = fs.read_dir_sync(path.as_path()) else {
    return storages;
  };

  // Cache directories are laid out as `<version>`.
  for version in versions {
    if version.starts_with(['.', '_']) {
      continue;
    }
    if !fs
      .metadata_sync(&path.join(&version))
      .is_ok_and(|metadata| metadata.is_directory)
    {
      continue;
    }

    let storage = create_storage(
      StorageOptions::FileSystem {
        directory: path.clone(),
      },
      version.clone(),
      None,
      None,
      fs.clone(),
    );

    storages.insert(version, storage);
  }

  storages
}

/// Compare cache dir from two directories and return whether they are equal
pub async fn compare_cache_dir(path1: Utf8PathBuf, path2: Utf8PathBuf) -> Result<()> {
  let cache_paths1 = find_relative_cache_path(&path1);
  let cache_paths2 = find_relative_cache_path(&path2);
  let debug_info = DebugInfo::default()
    .with_field("path1", path1.as_ref())
    .with_field("path2", path2.as_ref());

  // Check if versions are identical
  ensure_iter_equal(
    "Cache directories",
    cache_paths1.iter(),
    cache_paths2.iter(),
    &debug_info,
  )?;

  for cache_relative_path in &cache_paths1 {
    let cache_path1 = join_relative_cache_path(&path1, cache_relative_path);
    let cache_path2 = join_relative_cache_path(&path2, cache_relative_path);
    let debug_info = DebugInfo::default()
      .with_field("path1", cache_path1.as_ref())
      .with_field("path2", cache_path2.as_ref());

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

      let storage2 = storages2.remove(&version).expect("should have storage");

      compare_storage(storage1, storage2, cur_debug_info).await?;
    }
  }

  Ok(())
}

/// Compare two storages and return whether they are equal
async fn compare_storage(
  storage1: BoxStorage,
  storage2: BoxStorage,
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
      occasion::meta::SCOPE => {
        occasion::meta::compare(&*storage1, &*storage2, cur_debug_info).await?;
      }
      occasion::make::SCOPE => {
        occasion::make::compare(&*storage1, &*storage2, cur_debug_info).await?;
      }
      _ => {
        // TODO compare snapshot
        return Err(error!(
          "Comparison for unknown scope: {} \n{}",
          scope, debug_info
        ));
      }
    }
  }

  Ok(())
}
