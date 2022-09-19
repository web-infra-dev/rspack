use crate::RawOption;
use rspack_core::{CompilerOptionsBuilder, Devtool};

pub type RawDevtool = bool;

impl RawOption<Devtool> for RawDevtool {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<Devtool> {
    Ok(self)
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
