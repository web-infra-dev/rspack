use napi_derive::napi;
use rspack_core::CacheOptions;

#[derive(Debug, Default)]
#[napi(object, object_to_js = false)]
pub struct RawCacheOptions {
  pub r#type: String,
  pub max_generations: Option<u32>,
}

impl From<RawCacheOptions> for CacheOptions {
  fn from(value: RawCacheOptions) -> CacheOptions {
    let RawCacheOptions {
      r#type,
      max_generations,
    } = value;

    match r#type.as_str() {
      "memory" => CacheOptions::Memory { max_generations },
      _ => CacheOptions::Disabled,
    }
  }
}
