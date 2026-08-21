use napi_derive::napi;
use rspack_core::NewCacheOptions;

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawNewCache {
  pub code_generation: bool,
  pub minimize: bool,
}

impl From<RawNewCache> for NewCacheOptions {
  fn from(value: RawNewCache) -> Self {
    Self {
      code_generation: value.code_generation,
      minimize: value.minimize,
    }
  }
}
