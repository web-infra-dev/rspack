use crate::RawOption;
use rspack_core::{CompilerOptionsBuilder, Target};
use std::str::FromStr;

pub type RawTarget = String;

impl RawOption<Target> for RawTarget {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> Target {
    Target::from_str(&self).unwrap()
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    String::from("web")
  }
}
