/// snapshot
pub struct Snapshot {
  timestamp: u64,
  hash: String,
  file_path: String,
  managed_path: Vec<String>,
}

impl Snapshot {
  pub fn new() {}
  pub fn checkValid(&self) {}
}
