use napi_derive::napi;
use rspack_core::{CompilerOptionsBuilder, Experiments};
use serde::Deserialize;

use crate::RawOption;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawExperiments {
  pub lazy_compilation: bool,
  pub incremental_rebuild: bool,
}

impl RawOption<Experiments> for RawExperiments {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<Experiments> {
    Ok(Experiments {
      lazy_compilation: self.lazy_compilation,
      incremental_rebuild: self.incremental_rebuild,
    })
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
