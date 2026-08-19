use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_cacheable::cacheable;
use rspack_core::{CacheFacade, CacheValue, Etag, Resolver, loader_cache_item};
use rspack_error::Result;
use rspack_hash::{HashFunction, RspackHasher};
use rspack_loader_runner::DescriptionData;
use rspack_paths::Utf8Path;

#[cacheable]
#[derive(Clone)]
struct LoaderCacheEntry {
  content: Option<Vec<u8>>,
  content_is_string: bool,
  source_map: Option<Vec<u8>>,
}

#[napi(object)]
pub struct JsLoaderCacheEntry {
  pub content: Either<Null, Buffer>,
  pub content_is_string: bool,
  pub source_map: Option<Buffer>,
}

#[napi]
pub struct JsLoaderCache {
  cache: CacheFacade,
  module_identifier: String,
  loader_names: Vec<String>,
}

pub struct JsLoaderCacheObject(JsLoaderCache);

impl FromNapiValue for JsLoaderCacheObject {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let instance =
      unsafe { <ClassInstance<JsLoaderCache> as FromNapiValue>::from_napi_value(env, napi_val)? };
    Ok(Self(JsLoaderCache {
      cache: instance.cache.clone(),
      module_identifier: instance.module_identifier.clone(),
      loader_names: instance.loader_names.clone(),
    }))
  }
}

impl ToNapiValue for JsLoaderCacheObject {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    value: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    unsafe { ToNapiValue::to_napi_value(env, value.0) }
  }
}

impl TypeName for JsLoaderCacheObject {
  fn type_name() -> &'static str {
    "JsLoaderCache"
  }

  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

impl ValidateNapiValue for JsLoaderCacheObject {}

impl JsLoaderCache {
  pub fn new(cache: CacheFacade, module_identifier: String, loader_names: Vec<String>) -> Self {
    Self {
      cache,
      module_identifier,
      loader_names,
    }
  }

  fn loader_name(&self, loader_index: u32) -> napi::Result<&str> {
    self
      .loader_names
      .get(loader_index as usize)
      .map(String::as_str)
      .ok_or_else(|| napi::Error::from_reason(format!("Invalid loader index {loader_index}")))
  }
}

impl JsLoaderCacheObject {
  pub fn new(cache: CacheFacade, module_identifier: String, loader_names: Vec<String>) -> Self {
    Self(JsLoaderCache::new(cache, module_identifier, loader_names))
  }
}

pub(crate) async fn loader_cache_version(
  resolver: &Resolver,
  path: &Utf8Path,
  is_package_request: bool,
  description_data: Option<&DescriptionData>,
  enabled: bool,
) -> Result<Option<String>> {
  if !enabled {
    return Ok(None);
  }
  if is_package_request
    && let Some((name, version)) = description_data.and_then(|data| {
      let package = data.json();
      Some((
        package.get("name")?.as_str()?,
        package.get("version")?.as_str()?,
      ))
    })
  {
    return Ok(Some(format!("package:{name}@{version}")));
  }

  // V1 fingerprints only the resolved loader entry file. Files that the
  // loader imports or requires are intentionally not included yet.
  let contents = resolver.inner_fs().read(path).await?;
  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  hasher.write(&contents);
  Ok(Some(format!("file:{:016x}", hasher.finish())))
}

#[napi]
impl JsLoaderCache {
  #[napi]
  pub fn get(&self, loader_index: u32, etag: String) -> napi::Result<Option<JsLoaderCacheEntry>> {
    let loader_name = self.loader_name(loader_index)?;
    let item_cache = loader_cache_item(
      &self.cache,
      &self.module_identifier,
      loader_name,
      Etag::from(etag),
    );
    let Some(entry) = item_cache
      .get::<LoaderCacheEntry>()
      .map_err(|error| napi::Error::from_reason(error.to_string()))?
    else {
      return Ok(None);
    };

    Ok(Some(JsLoaderCacheEntry {
      content: entry
        .content
        .clone()
        .map_or(Either::A(Null), |content| Either::B(content.into())),
      content_is_string: entry.content_is_string,
      source_map: entry.source_map.clone().map(Into::into),
    }))
  }

  #[napi]
  pub fn store(
    &self,
    loader_index: u32,
    etag: String,
    output: JsLoaderCacheEntry,
  ) -> napi::Result<()> {
    let loader_name = self.loader_name(loader_index)?;
    let entry = LoaderCacheEntry {
      content: match output.content {
        Either::A(_) => None,
        Either::B(content) => Some(content.to_vec()),
      },
      content_is_string: output.content_is_string,
      source_map: output.source_map.map(|source_map| source_map.to_vec()),
    };
    let item_cache = loader_cache_item(
      &self.cache,
      &self.module_identifier,
      loader_name,
      Etag::from(etag),
    );
    item_cache
      .store(CacheValue::new(entry))
      .map_err(|error| napi::Error::from_reason(error.to_string()))
  }
}
