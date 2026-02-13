#[derive(Debug, Clone)]
pub struct DBOptions {
  pub page_count: usize,
  pub max_pack_size: usize,
}

impl Default for DBOptions {
  fn default() -> Self {
    Self {
      page_count: 10,
      max_pack_size: 512 * 1024, // 512KB
    }
  }
}
