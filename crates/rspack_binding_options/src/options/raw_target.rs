use crate::RawOption;
use rspack_core::{CompilerOptionsBuilder, Target};
use std::str::FromStr;

pub type RawTarget = String;

impl RawOption<Target> for RawTarget {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<Target> {
    Target::from_str(&self)
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    String::from("web")
  }
}
