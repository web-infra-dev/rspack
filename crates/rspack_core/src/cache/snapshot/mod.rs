use std::{path::PathBuf, time::SystemTime};

use rustc_hash::FxHashMap as HashMap;

mod manager;
pub use manager::SnapshotManager;

/// Snapshot store dependenct files update time and hash
#[derive(Debug, Clone)]
pub struct Snapshot {
  pub file_update_times: HashMap<PathBuf, SystemTime>,
  pub file_hashes: HashMap<PathBuf, u64>,
}
