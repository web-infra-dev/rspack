use hashbrown::HashMap;
use std::time::SystemTime;

mod manager;
pub use manager::SnapshotManager;

/// Snapshot store dependenct files update time and hash
#[derive(Debug, Clone)]
pub struct Snapshot {
  pub file_update_times: HashMap<String, SystemTime>,
  pub file_hashs: HashMap<String, u64>,
}
