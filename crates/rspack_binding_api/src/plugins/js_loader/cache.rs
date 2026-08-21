use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_cacheable::cacheable;
use rspack_core::{
  CacheFacade, CacheValue, Content, Etag, Resolver, loader_cache_etag, loader_cache_item,
};
use rspack_error::Result;
use rspack_hash::{HashFunction, RspackHasher};
use rspack_loader_runner::{DescriptionData, LoaderRunnerOptions};
use rspack_paths::Utf8Path;

#[cacheable]
#[derive(Clone)]
struct LoaderCacheEntry {
  content: Option<Vec<u8>>,
  content_is_string: bool,
  source_map: Option<Vec<u8>>,
  parse_meta: String,
}

#[napi(object)]
pub struct JsLoaderCacheEntry {
  pub content: Either<Null, Uint8Array>,
  pub content_is_string: bool,
  pub source_map: Option<Uint8Array>,
  pub parse_meta: HashMap<String, String>,
}

#[napi]
pub struct JsLoaderCache {
  cache: CacheFacade,
  module_identifier: String,
  loaders: Vec<LoaderRunnerOptions>,
  pending_etags: Arc<Mutex<Vec<Option<Etag>>>>,
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
      loaders: instance.loaders.clone(),
      pending_etags: instance.pending_etags.clone(),
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
  fn new(cache: CacheFacade, module_identifier: String, loaders: Vec<LoaderRunnerOptions>) -> Self {
    let pending_etags = Arc::new(Mutex::new(vec![None; loaders.len()]));
    Self {
      cache,
      module_identifier,
      loaders,
      pending_etags,
    }
  }

  fn loader(&self, loader_index: u32) -> napi::Result<&LoaderRunnerOptions> {
    self
      .loaders
      .get(loader_index as usize)
      .ok_or_else(|| napi::Error::from_reason(format!("Invalid loader index {loader_index}")))
  }

  fn set_pending_etag(&self, loader_index: u32, etag: Option<Etag>) -> napi::Result<()> {
    let mut pending_etags = self
      .pending_etags
      .lock()
      .map_err(|_| napi::Error::from_reason("Loader cache state is poisoned"))?;
    let pending_etag = pending_etags
      .get_mut(loader_index as usize)
      .ok_or_else(|| napi::Error::from_reason(format!("Invalid loader index {loader_index}")))?;
    *pending_etag = etag;
    Ok(())
  }

  fn take_pending_etag(&self, loader_index: u32) -> napi::Result<Option<Etag>> {
    let mut pending_etags = self
      .pending_etags
      .lock()
      .map_err(|_| napi::Error::from_reason("Loader cache state is poisoned"))?;
    Ok(
      pending_etags
        .get_mut(loader_index as usize)
        .ok_or_else(|| napi::Error::from_reason(format!("Invalid loader index {loader_index}")))?
        .take(),
    )
  }
}

impl JsLoaderCacheObject {
  pub(super) fn new(
    cache: CacheFacade,
    module_identifier: String,
    loaders: Vec<LoaderRunnerOptions>,
  ) -> Self {
    Self(JsLoaderCache::new(cache, module_identifier, loaders))
  }
}

pub(crate) async fn loader_cache_version(
  resolver: &Resolver,
  path: &Utf8Path,
  is_package_request: bool,
  description_data: Option<&DescriptionData>,
) -> Result<Option<String>> {
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
  pub fn get(
    &self,
    loader_index: u32,
    content: Uint8Array,
    source_map: Option<Uint8Array>,
  ) -> napi::Result<Option<JsLoaderCacheEntry>> {
    let loader = self.loader(loader_index)?;
    let etag = loader_cache_etag(
      &Content::Buffer(content.to_vec()),
      source_map.as_deref(),
      &loader.options_cache_key,
      &loader.loader_version,
    );
    let item_cache = loader_cache_item(
      &self.cache,
      &self.module_identifier,
      &loader.loader_name,
      etag.clone(),
    );
    let Some(entry) = item_cache
      .get::<LoaderCacheEntry>()
      .map_err(|error| napi::Error::from_reason(error.to_string()))?
    else {
      self.set_pending_etag(loader_index, Some(etag))?;
      return Ok(None);
    };
    self.set_pending_etag(loader_index, None)?;

    Ok(Some(JsLoaderCacheEntry {
      content: entry
        .content
        .clone()
        .map_or(Either::A(Null), |content| Either::B(content.into())),
      content_is_string: entry.content_is_string,
      source_map: entry.source_map.clone().map(Into::into),
      parse_meta: serde_json::from_str(&entry.parse_meta)
        .map_err(|error| napi::Error::from_reason(error.to_string()))?,
    }))
  }

  #[napi]
  pub fn store(&self, loader_index: u32, output: JsLoaderCacheEntry) -> napi::Result<()> {
    let loader_name = &self.loader(loader_index)?.loader_name;
    let Some(etag) = self.take_pending_etag(loader_index)? else {
      return Ok(());
    };
    let entry = LoaderCacheEntry {
      content: match output.content {
        Either::A(_) => None,
        Either::B(content) => Some(content.to_vec()),
      },
      content_is_string: output.content_is_string,
      source_map: output.source_map.map(|source_map| source_map.to_vec()),
      parse_meta: serde_json::to_string(&output.parse_meta)
        .map_err(|error| napi::Error::from_reason(error.to_string()))?,
    };
    let item_cache = loader_cache_item(&self.cache, &self.module_identifier, loader_name, etag);
    item_cache
      .store(CacheValue::new(entry))
      .map_err(|error| napi::Error::from_reason(error.to_string()))
  }
}
