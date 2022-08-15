use super::RawOption;
use rspack_core::{BundleEntries, CompilerOptionsBuilder};
use std::collections::HashMap;

pub type RawEntry = HashMap<String, String>;

impl RawOption<BundleEntries> for RawEntry {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> BundleEntries {
    self
      .into_iter()
      .map(|(name, src)| (name, src.into()))
      .collect()
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
