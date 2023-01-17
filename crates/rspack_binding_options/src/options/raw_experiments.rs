use napi_derive::napi;
use rspack_core::{CompilerOptionsBuilder, Experiments};
use serde::Deserialize;

use crate::RawOption;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawExperiments {
  pub lazy_compilation: bool,
  pub changed_hmr: bool,
}

impl RawOption<Experiments> for RawExperiments {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<Experiments> {
    Ok(Experiments {
      lazy_compilation: self.lazy_compilation,
      changed_hmr: self.changed_hmr,
    })
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
