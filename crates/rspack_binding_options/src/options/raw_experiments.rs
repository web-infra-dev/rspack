use napi_derive::napi;
use rspack_core::Experiments;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawExperiments {
  pub lazy_compilation: bool,
  pub incremental_rebuild: bool,
  pub async_web_assembly: bool,
  pub new_split_chunks: bool,
  pub css: bool,
}

impl From<RawExperiments> for Experiments {
  fn from(value: RawExperiments) -> Self {
    Self {
      lazy_compilation: value.lazy_compilation,
      incremental_rebuild: value.incremental_rebuild,
      async_web_assembly: value.async_web_assembly,
      new_split_chunks: value.new_split_chunks,
      css: value.css,
    }
  }
}
