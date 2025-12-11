mod build_dependencies;
mod debug_info;
mod occasion;
mod snapshot;
mod utils;

use std::sync::Arc;

use debug_info::DebugInfo;
use rspack_core::cache::persistent::storage::Storage;
use rspack_error::{Result, error};
use rspack_paths::Utf8PathBuf;
use utils::{ensure_sets_equal, load_storages_from_path};

/// Compare two cache directories and return whether they are equal
pub async fn compare_storage_dirs(path1: Utf8PathBuf, path2: Utf8PathBuf) -> Result<()> {
  let debug_info = DebugInfo::default()
    .with_field("path1", &path1.to_string())
    .with_field("path2", &path2.to_string());

  // Load storages from both paths
  let storages1 = load_storages_from_path(&path1)?;
  let mut storages2 = load_storages_from_path(&path2)?;

  // Check if versions are identical
  ensure_sets_equal(
    "Version directory",
    storages1.keys(),
    storages2.keys(),
    &debug_info,
  )?;

  // Compare storages for each version
  for (version, storage1) in storages1 {
    let cur_debug_info = debug_info.with_field("version", &version);

    let storage2 = storages2.remove(&version).unwrap();

    compare(storage1, storage2, cur_debug_info).await?;
  }

  Ok(())
}

async fn compare(
  storage1: Arc<dyn Storage>,
  storage2: Arc<dyn Storage>,
  debug_info: DebugInfo,
) -> Result<()> {
  // Get scopes from both storages
  let scopes1 = storage1.scopes().await?;
  let scopes2 = storage2.scopes().await?;

  // Check if scopes are identical
  ensure_sets_equal("Scope", scopes1.iter(), scopes2.iter(), &debug_info)?;

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
