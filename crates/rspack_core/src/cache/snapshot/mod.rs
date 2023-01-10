use std::{path::PathBuf, time::SystemTime};

use hashbrown::HashMap;

mod manager;
pub use manager::SnapshotManager;

/// Snapshot store dependenct files update time and hash
#[derive(Debug, Clone)]
pub struct Snapshot {
  pub file_update_times: HashMap<PathBuf, SystemTime>,
  pub file_hashs: HashMap<PathBuf, u64>,
}
