use crate::BundleMode;

pub type ResolveOption = nodejs_resolver::ResolverOptions;

impl From<BundleMode> for ResolveOption {
  fn from(_: BundleMode) -> Self {
    Self {
      extensions: vec![".tsx", ".jsx", ".ts", ".js", ".json"]
        .into_iter()
        .map(|s| s.to_string())
        .collect(),
      alias_fields: vec![String::from("browser")],
      ..Default::default()
    }
  }
}
