use std::sync::Arc;

use rspack_cacheable::from_bytes;
pub use rspack_core::cache::persistent::build_dependencies::SCOPE;
use rspack_core::cache::persistent::{snapshot::Strategy, storage::Storage};
use rspack_error::Result;
use rustc_hash::FxHashMap as HashMap;

use crate::{debug_info::DebugInfo, utils::ensure_iter_equal};

/// Compare build_dependencies scope data between two storages
pub async fn compare(
  storage1: Arc<dyn Storage>,
  storage2: Arc<dyn Storage>,
  debug_info: DebugInfo,
) -> Result<()> {
  // Load build_dependencies data from both storages
  let data1 = storage1.load(SCOPE).await?;
  let data2 = storage2.load(SCOPE).await?;

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
  ensure_iter_equal(
    "Build dependency path",
    map1.keys(),
    map2.keys(),
    &debug_info,
  )?;

  // Compare strategies for each path
  for (key, strategy1) in &map1 {
    let strategy2 = map2.get(key).unwrap();

    if strategy1 != strategy2 {
      let path_str = String::from_utf8_lossy(key);
      let mut error_msg = format!(
        "Build dependency strategy mismatch for path: {}\n",
        path_str
      );
      error_msg.push_str(&format!("  storage1: {:?}\n", strategy1));
      error_msg.push_str(&format!("  storage2: {:?}\n", strategy2));
      error_msg.push_str(&debug_info.to_string());
      return Err(rspack_error::error!(error_msg));
    }
  }

  Ok(())
}
