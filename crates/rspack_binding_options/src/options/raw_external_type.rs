use crate::RawOption;
use rspack_core::{CompilerOptionsBuilder, ExternalType};
use std::str::FromStr;

pub type RawExternalType = String;

impl RawOption<ExternalType> for RawExternalType {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<ExternalType> {
    ExternalType::from_str(&self)
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
