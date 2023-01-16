use napi_derive::napi;
use rspack_core::{
  CacheOptions, CompilerOptionsBuilder, FileSystemCacheOptions, MemoryCacheOptions,
};
use serde::Deserialize;

use crate::RawOption;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawCacheOptions {
  pub r#type: String,
  pub max_generations: u32,
  pub max_age: u32,
  pub profile: bool,
  pub build_dependencies: Vec<String>,
  pub cache_directory: String,
  pub cache_location: String,
  pub name: String,
  pub version: String,
}

impl RawOption<CacheOptions> for RawCacheOptions {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<CacheOptions> {
    let Self {
      r#type,
      max_generations,
      max_age,
      profile,
      build_dependencies,
      cache_directory,
      cache_location,
      name,
      version,
    } = self;

    Ok(match r#type.as_str() {
      "memory" => CacheOptions::Memory(MemoryCacheOptions { max_generations }),
      "filesystem" => CacheOptions::FileSystem(FileSystemCacheOptions {
        max_age,
        profile,
        build_dependencies,
        cache_directory,
        cache_location,
        name,
        version,
      }),
      _ => CacheOptions::Disabled,
    })
  }

  fn fallback_value(_: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
