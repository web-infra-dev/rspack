use std::str::FromStr;

use crate::RawOption;
use rspack_core::{CompilerOptionsBuilder, Devtool};

pub type RawDevtool = String;

impl RawOption<Devtool> for RawDevtool {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<Devtool> {
    Devtool::from_str(&self)
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
