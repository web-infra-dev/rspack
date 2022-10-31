use super::RawOption;
use rspack_core::{BundleEntries, CompilerOptionsBuilder};
use std::collections::HashMap;

pub type RawEntry = HashMap<String, Vec<String>>;

impl RawOption<BundleEntries> for RawEntry {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<BundleEntries> {
    Ok(
      self
        .into_iter()
        .map(|(name, arr)| (name, arr.into_iter().map(|src| src.into()).collect()))
        .collect(),
    )
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
