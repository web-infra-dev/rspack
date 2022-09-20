use crate::RawOption;
use rspack_core::{CompilerOptionsBuilder, ExternalType};

pub type RawExternalType = String;

impl RawOption<ExternalType> for RawExternalType {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<ExternalType> {
    ExternalType::new(self)
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
