use std::sync::Arc;

use crate::{BundleContext, ResolvedId};

#[derive(Debug)]
struct Bundle {
  enties: Vec<String>,
  pub context: Arc<BundleContext>,
}

impl Bundle {
  pub fn add_entry(&mut self, entry: String) {
    self.enties.push(entry);
  }

  pub async fn build(&mut self) {}
}
