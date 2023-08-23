use napi_derive::napi;
use rspack_core::PresetEnv;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawPresetEnv {
  pub targets: Vec<String>,
  #[napi(ts_type = "'usage' | 'entry'")]
  pub mode: Option<String>,
  pub core_js: Option<String>,
}

impl From<RawPresetEnv> for PresetEnv {
  fn from(raw_preset_env: RawPresetEnv) -> Self {
    Self {
      targets: raw_preset_env.targets,
      mode: raw_preset_env.mode.and_then(|mode| match mode.as_str() {
        "usage" => Some(swc_core::ecma::preset_env::Mode::Usage),
        "entry" => Some(swc_core::ecma::preset_env::Mode::Entry),
        _ => None,
      }),
      core_js: raw_preset_env.core_js,
    }
  }
}
