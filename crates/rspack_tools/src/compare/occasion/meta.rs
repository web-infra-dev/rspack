use std::sync::Arc;

use rspack_cacheable::{cacheable, from_bytes};
pub(crate) use rspack_core::cache::persistent::occasion::meta::SCOPE;
use rspack_core::cache::persistent::storage::Storage;
use rspack_error::Result;

use crate::debug_info::DebugInfo;

/// Meta struct that mirrors rspack_core's Meta structure
#[cacheable]
#[derive(Debug)]
struct Meta {
  pub max_dependencies_id: u32,
}

/// Compare meta scope data between two storages
pub(crate) async fn compare(
  storage1: Arc<dyn Storage>,
  storage2: Arc<dyn Storage>,
  debug_info: DebugInfo,
) -> Result<()> {
  // Load meta data from both storages
  let data1 = storage1.load(SCOPE).await?;
  let data2 = storage2.load(SCOPE).await?;

  // Meta scope should have exactly one entry with key "default"
  if data1.len() != 1 {
    return Err(rspack_error::error!(
      "Expected exactly one meta entry in storage1, got {}\n{}",
      data1.len(),
      debug_info
    ));
  }

  if data2.len() != 1 {
    return Err(rspack_error::error!(
      "Expected exactly one meta entry in storage2, got {}\n{}",
      data2.len(),
      debug_info
    ));
  }

  // Deserialize Meta from both storages
  let (_, value1) = &data1[0];
  let (_, value2) = &data2[0];

  let _meta1: Meta = from_bytes::<Meta, ()>(value1, &()).expect("should deserialize meta");
  let _meta2: Meta = from_bytes::<Meta, ()>(value2, &()).expect("should deserialize meta");

  // Compare Meta fields
  // Note: We skip comparing max_dependencies_id as it's an internal counter
  // that may differ between builds but doesn't affect cache correctness.
  // The dependency IDs are unique within each build and are regenerated during cache recovery.

  // If Meta struct gets more fields in the future, add comparisons here:
  // if _meta1.some_field != _meta2.some_field {
  //   let mut error_msg = format!("Meta field 'some_field' mismatch\n");
  //   error_msg.push_str(&format!("  storage1: {:?}\n", _meta1.some_field));
  //   error_msg.push_str(&format!("  storage2: {:?}\n", _meta2.some_field));
  //   error_msg.push_str(&debug_info.to_string());
  //   return Err(rspack_error::error!(error_msg));
  // }

  // Commented out: dependency_id comparison (kept for reference)
  // if _meta1.max_dependencies_id != _meta2.max_dependencies_id {
  //   let mut error_msg = format!("Meta max_dependencies_id mismatch\n");
  //   error_msg.push_str(&format!("  storage1: {}\n", _meta1.max_dependencies_id));
  //   error_msg.push_str(&format!("  storage2: {}\n", _meta2.max_dependencies_id));
  //   error_msg.push_str(&debug_info.to_string());
  //   return Err(rspack_error::error!(error_msg));
  // }

  Ok(())
}
