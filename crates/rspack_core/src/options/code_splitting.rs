use crate::BundleMode;

#[derive(Debug)]
pub struct CodeSplittingOptions {
  pub enable: bool,
  pub reuse_existing_chunk: bool,
}

impl Default for CodeSplittingOptions {
  fn default() -> Self {
    Self {
      enable: true,
      reuse_existing_chunk: true,
    }
  }
}

impl From<BundleMode> for CodeSplittingOptions {
  fn from(mode: BundleMode) -> Self {
    Self {
      enable: !mode.is_none(),
      reuse_existing_chunk: !mode.is_none(),
    }
  }
}
