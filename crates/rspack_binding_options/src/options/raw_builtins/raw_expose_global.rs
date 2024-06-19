use napi_derive::napi;

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawExposeGlobalPluginOptions {
  pub global: String,
}
