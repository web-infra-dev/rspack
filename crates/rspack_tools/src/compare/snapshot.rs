use std::sync::Arc;

use rspack_cacheable::from_bytes;
use rspack_core::cache::persistent::{
  snapshot::{SnapshotScope, Strategy},
  storage::Storage,
};
use rspack_error::Result;
use rustc_hash::FxHashMap as HashMap;

use crate::{debug_info::DebugInfo, utils::ensure_iter_equal};

/// Compare snapshot scope data between two storages
#[allow(dead_code)]
pub(super) async fn compare(
  scope: SnapshotScope,
  storage1: Arc<dyn Storage>,
  storage2: Arc<dyn Storage>,
  debug_info: DebugInfo,
) -> Result<()> {
  // Load snapshot data from both storages
  // TODO check SnapshotScope::Context && SnapshotScope::Missing
  let data1 = storage1.load(scope.name()).await?;
  let data2 = storage2.load(scope.name()).await?;

  // Convert to HashMap for easier comparison
  let map1: HashMap<_, _> = data1
    .into_iter()
    .map(|(key, value)| {
      let strategy: Strategy =
        from_bytes::<Strategy, ()>(&value, &()).expect("should deserialize strategy");
      (key, strategy)
    })
    .collect();

  let map2: HashMap<_, _> = data2
    .into_iter()
    .map(|(key, value)| {
      let strategy: Strategy =
        from_bytes::<Strategy, ()>(&value, &()).expect("should deserialize strategy");
      (key, strategy)
    })
    .collect();

  // Check if keys are identical
  ensure_iter_equal("Snapshot path", map1.keys(), map2.keys(), &debug_info)?;

  // Compare strategies for each path
  for (key, strategy1) in &map1 {
    let strategy2 = map2.get(key).expect("should have strategy");

    if strategy1 != strategy2 {
      let path_str = String::from_utf8_lossy(key);
      let mut error_msg = format!("Snapshot strategy mismatch for path: {path_str}\n");
      error_msg.push_str(&format!("  storage1: {strategy1:?}\n"));
      error_msg.push_str(&format!("  storage2: {strategy2:?}\n"));
      error_msg.push_str(&debug_info.to_string());
      return Err(rspack_error::error!(error_msg));
    }
  }

  Ok(())
}
