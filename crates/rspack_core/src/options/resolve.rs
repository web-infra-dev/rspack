use std::collections::HashSet;

use crate::BundleMode;

#[derive(Debug, Clone)]
pub struct ResolveOption {
  pub extensions: Vec<String>,
  pub alias: Vec<(String, Option<String>)>,
  pub condition_names: HashSet<String>,
  pub symlinks: bool,
  pub alias_field: String,
}

impl Default for ResolveOption {
  fn default() -> Self {
    Self {
      extensions: vec![".tsx", ".jsx", ".ts", ".js", ".json"]
        .into_iter()
        .map(|s| s.to_string())
        .collect(),
      alias: vec![],
      condition_names: Default::default(),
      symlinks: true,
      alias_field: String::from("browser"),
    }
  }
}

impl From<BundleMode> for ResolveOption {
  fn from(_: BundleMode) -> Self {
    Self::default()
  }
}
