use std::str::FromStr;

use rspack_core::{CompilerOptionsBuilder, ExternalType};

use crate::RawOption;

pub type RawExternalType = String;

impl RawOption<ExternalType> for RawExternalType {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<ExternalType> {
    ExternalType::from_str(&self)
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
