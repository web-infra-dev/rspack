use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct PackOptions {
  pub bucket_size: usize,
  pub pack_size: usize,
  pub expire: u64,
}

impl PackOptions {
  pub fn is_expired(&self, last_modified: &u64) -> bool {
    let current_time = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("get current time failed")
      .as_millis() as u64;
    current_time - last_modified > self.expire
  }
}
