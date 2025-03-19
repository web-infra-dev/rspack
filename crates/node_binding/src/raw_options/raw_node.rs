use napi_derive::napi;

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawNodeOption {
  pub dirname: String,
  pub filename: String,
  pub global: String,
}
