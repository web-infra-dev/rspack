#[derive(Debug)]
pub struct Options {
  pub max_pack_size: usize,
}

impl Default for Options {
  fn default() -> Self {
    Self {
      max_pack_size: 512 * 1024, // 512KB
    }
  }
}
