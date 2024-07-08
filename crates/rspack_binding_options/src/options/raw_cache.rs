use napi_derive::napi;
use rspack_core::{CacheOptions, FileSystemCacheOptions, MemoryCacheOptions};

#[derive(Debug, Default)]
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

impl From<RawCacheOptions> for CacheOptions {
  fn from(value: RawCacheOptions) -> CacheOptions {
    let RawCacheOptions {
      r#type,
      max_generations,
      max_age,
      profile,
      build_dependencies,
      cache_directory,
      cache_location,
      name,
      version,
    } = value;

    match r#type.as_str() {
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
    }
  }
}
