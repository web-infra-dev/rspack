use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_cacheable::cacheable;
use rspack_core::{CacheValue, Etag, LoaderCache, Resolver};
use rspack_error::Result;
use rspack_hash::{HashFunction, RspackHasher};
use rspack_loader_runner::DescriptionData;
use rspack_paths::Utf8Path;
use rustc_hash::FxHashMap as HashMap;

#[cacheable]
#[derive(Clone, Default)]
struct DependencyDelta {
  added: Vec<String>,
  removed: Vec<String>,
}

#[cacheable]
#[derive(Clone, Default)]
struct ParseMetaDelta {
  upserted: Vec<(String, String)>,
  removed: Vec<String>,
}

#[cacheable]
#[derive(Clone)]
struct LoaderCacheEntry {
  content: Option<Vec<u8>>,
  content_is_string: bool,
  source_map: Option<Vec<u8>>,
  additional_data: Option<Vec<u8>>,
  file_dependencies: DependencyDelta,
  context_dependencies: DependencyDelta,
  missing_dependencies: DependencyDelta,
  build_dependencies: DependencyDelta,
  parse_meta: ParseMetaDelta,
}

#[napi(object)]
pub struct JsLoaderCacheEntry {
  pub content: Either<Null, Buffer>,
  pub content_is_string: bool,
  pub source_map: Option<Buffer>,
  pub additional_data: Option<Buffer>,
  pub file_dependencies_added: Vec<String>,
  pub file_dependencies_removed: Vec<String>,
  pub context_dependencies_added: Vec<String>,
  pub context_dependencies_removed: Vec<String>,
  pub missing_dependencies_added: Vec<String>,
  pub missing_dependencies_removed: Vec<String>,
  pub build_dependencies_added: Vec<String>,
  pub build_dependencies_removed: Vec<String>,
  pub parse_meta_upserted: HashMap<String, String>,
  pub parse_meta_removed: Vec<String>,
}

#[napi]
pub struct JsLoaderCache {
  cache: Arc<LoaderCache>,
  module_identifier: String,
  loader_keys: Vec<String>,
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
      cache: Arc::clone(&instance.cache),
      module_identifier: instance.module_identifier.clone(),
      loader_keys: instance.loader_keys.clone(),
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
  pub fn new(cache: Arc<LoaderCache>, module_identifier: String, loader_keys: Vec<String>) -> Self {
    Self {
      cache,
      module_identifier,
      loader_keys,
    }
  }

  fn loader_key(&self, loader_index: u32) -> napi::Result<&str> {
    self
      .loader_keys
      .get(loader_index as usize)
      .map(String::as_str)
      .ok_or_else(|| napi::Error::from_reason(format!("Invalid loader index {loader_index}")))
  }
}

impl JsLoaderCacheObject {
  pub fn new(cache: Arc<LoaderCache>, module_identifier: String, loader_keys: Vec<String>) -> Self {
    Self(JsLoaderCache::new(cache, module_identifier, loader_keys))
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
    let cache_key = self.loader_key(loader_index)?;
    let item_cache = self
      .cache
      .cache_item(cache_key, &self.module_identifier, Etag::from(etag));
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
      additional_data: entry.additional_data.clone().map(Into::into),
      file_dependencies_added: entry.file_dependencies.added.clone(),
      file_dependencies_removed: entry.file_dependencies.removed.clone(),
      context_dependencies_added: entry.context_dependencies.added.clone(),
      context_dependencies_removed: entry.context_dependencies.removed.clone(),
      missing_dependencies_added: entry.missing_dependencies.added.clone(),
      missing_dependencies_removed: entry.missing_dependencies.removed.clone(),
      build_dependencies_added: entry.build_dependencies.added.clone(),
      build_dependencies_removed: entry.build_dependencies.removed.clone(),
      parse_meta_upserted: entry.parse_meta.upserted.iter().cloned().collect(),
      parse_meta_removed: entry.parse_meta.removed.clone(),
    }))
  }

  #[napi]
  pub fn store(
    &self,
    loader_index: u32,
    etag: String,
    output: JsLoaderCacheEntry,
  ) -> napi::Result<()> {
    let cache_key = self.loader_key(loader_index)?;
    let entry = LoaderCacheEntry {
      content: match output.content {
        Either::A(_) => None,
        Either::B(content) => Some(content.to_vec()),
      },
      content_is_string: output.content_is_string,
      source_map: output.source_map.map(|source_map| source_map.to_vec()),
      additional_data: output
        .additional_data
        .map(|additional_data| additional_data.to_vec()),
      file_dependencies: DependencyDelta {
        added: output.file_dependencies_added,
        removed: output.file_dependencies_removed,
      },
      context_dependencies: DependencyDelta {
        added: output.context_dependencies_added,
        removed: output.context_dependencies_removed,
      },
      missing_dependencies: DependencyDelta {
        added: output.missing_dependencies_added,
        removed: output.missing_dependencies_removed,
      },
      build_dependencies: DependencyDelta {
        added: output.build_dependencies_added,
        removed: output.build_dependencies_removed,
      },
      parse_meta: ParseMetaDelta {
        upserted: output.parse_meta_upserted.into_iter().collect(),
        removed: output.parse_meta_removed,
      },
    };
    let item_cache = self
      .cache
      .cache_item(cache_key, &self.module_identifier, Etag::from(etag));
    item_cache
      .store(CacheValue::new(entry))
      .map_err(|error| napi::Error::from_reason(error.to_string()))
  }
}
