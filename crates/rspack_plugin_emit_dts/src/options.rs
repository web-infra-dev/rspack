use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SwcDtsEmitOptions {
  pub root_dir: String,
  pub include: String,
  pub out_dir: String,
  pub abort_on_error: bool,
  pub emit: bool,
}

#[derive(Default, Deserialize, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct RawSwcDtsEmitOptions {
  pub root_dir: String,
  pub include: String,
  pub out_dir: String,
  pub abort_on_error: Option<bool>,
  pub emit: Option<bool>,
}

impl From<RawSwcDtsEmitOptions> for SwcDtsEmitOptions {
  fn from(value: RawSwcDtsEmitOptions) -> Self {
    Self {
      abort_on_error: value.abort_on_error.unwrap_or(true),
      emit: value.emit.unwrap_or(true),
      root_dir: value.root_dir,
      include: value.include,
      out_dir: value.out_dir,
    }
  }
}

#[derive(Debug)]
pub struct SwcDtsEmitRspackPluginOptions {
  pub extension: String,
}
