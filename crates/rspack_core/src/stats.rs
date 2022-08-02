use crate::Asset;

#[derive(Debug)]
pub struct Stats {
  assets: Box<[Asset]>,
}

impl Stats {
  pub fn new(assets: Box<[Asset]>) -> Self {
    Self { assets }
  }
}

impl Stats {
  pub fn assets(&self) -> &[Asset] {
    &self.assets
  }
}
