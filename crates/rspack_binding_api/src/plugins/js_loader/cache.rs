use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_cacheable::{
  cacheable,
  utils::PortablePath,
  with::{As, AsVec},
};
use rspack_core::{CacheValue, Etag, LoaderCache, Resolver};
use rspack_error::Result;
use rspack_hash::{HashFunction, RspackHasher};
use rspack_loader_runner::DescriptionData;
use rspack_napi::threadsafe_js_value_ref::ThreadsafeJsValueRef;
use rspack_paths::Utf8Path;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

#[cacheable]
#[derive(Clone, Default)]
struct DependencyDelta {
  #[cacheable(with=AsVec<As<PortablePath>>)]
  added: Vec<String>,
  #[cacheable(with=AsVec<As<PortablePath>>)]
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
  additional_data_cache_key: Option<String>,
  file_dependencies: DependencyDelta,
  context_dependencies: DependencyDelta,
  missing_dependencies: DependencyDelta,
  build_dependencies: DependencyDelta,
  parse_meta: ParseMetaDelta,
  requires_sidecar: bool,
}

#[derive(Clone)]
struct LoaderCacheSidecar {
  additional_data: ThreadsafeJsValueRef<Unknown<'static>>,
}

#[napi(object)]
pub struct JsLoaderCacheData {
  pub content: Either<Null, Buffer>,
  pub content_is_string: bool,
  pub source_map: Option<Buffer>,
  #[napi(ts_type = "any")]
  pub additional_data: Option<ThreadsafeJsValueRef<Unknown<'static>>>,
  pub additional_data_cache_key: Option<String>,
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub build_dependencies: Vec<String>,
  pub parse_meta: HashMap<String, String>,
  pub cacheable: bool,
  pub has_unhandled_side_effects: bool,
}

#[napi]
pub struct JsLoaderCache {
  cache: Arc<LoaderCache>,
  module_identifier: String,
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
  pub fn new(cache: Arc<LoaderCache>, module_identifier: String) -> Self {
    Self {
      cache,
      module_identifier,
    }
  }
}

impl JsLoaderCacheObject {
  pub fn new(cache: Arc<LoaderCache>, module_identifier: String) -> Self {
    Self(JsLoaderCache::new(cache, module_identifier))
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

fn dependency_delta(baseline: &[String], current: &[String]) -> DependencyDelta {
  let baseline = baseline.iter().cloned().collect::<HashSet<_>>();
  let current = current.iter().cloned().collect::<HashSet<_>>();
  DependencyDelta {
    added: current.difference(&baseline).cloned().collect(),
    removed: baseline.difference(&current).cloned().collect(),
  }
}

fn replay_dependency_delta(dependencies: &mut Vec<String>, delta: &DependencyDelta) {
  dependencies.retain(|dependency| !delta.removed.contains(dependency));
  dependencies.extend(delta.added.iter().cloned());
}

fn parse_meta_delta(
  baseline: &HashMap<String, String>,
  current: &HashMap<String, String>,
) -> ParseMetaDelta {
  ParseMetaDelta {
    upserted: current
      .iter()
      .filter(|(key, value)| baseline.get(*key) != Some(*value))
      .map(|(key, value)| (key.clone(), value.clone()))
      .collect(),
    removed: baseline
      .keys()
      .filter(|key| !current.contains_key(*key))
      .cloned()
      .collect(),
  }
}

fn replay_parse_meta(parse_meta: &mut HashMap<String, String>, delta: &ParseMetaDelta) {
  parse_meta.retain(|key, _| !delta.removed.contains(key));
  parse_meta.extend(delta.upserted.clone());
}

fn input_etag(data: &JsLoaderCacheData) -> Option<Etag> {
  if !data.parse_meta.is_empty() {
    return None;
  }
  let content = match &data.content {
    Either::A(_) => return None,
    Either::B(content) => content,
  };
  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  if data.content_is_string {
    hasher.write(b"string");
  } else {
    hasher.write(b"buffer");
  }
  hasher.write(content);
  if let Some(source_map) = &data.source_map {
    hasher.write(b"source-map");
    hasher.write(source_map);
  }
  if data.additional_data.is_some() {
    let additional_data_cache_key = data.additional_data_cache_key.as_ref()?;
    hasher.write(b"additional-data");
    hasher.write(additional_data_cache_key.as_bytes());
  }
  Some(Etag::from(format!("{:016x}", hasher.finish())))
}

fn output_additional_data_cache_key(
  cache_key: &str,
  etag: &Etag,
  output: &JsLoaderCacheData,
) -> Option<String> {
  output.additional_data.as_ref()?;
  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  hasher.write(cache_key.as_bytes());
  hasher.write(etag.as_str().as_bytes());
  Some(format!("{:016x}", hasher.finish()))
}

#[napi]
impl JsLoaderCache {
  #[napi]
  pub fn get(
    &self,
    cache_key: String,
    mut input: JsLoaderCacheData,
  ) -> napi::Result<Option<JsLoaderCacheData>> {
    if !input.cacheable || input.has_unhandled_side_effects {
      return Ok(None);
    }
    let Some(etag) = input_etag(&input) else {
      return Ok(None);
    };
    let (storage_key, item_cache) =
      self
        .cache
        .cache_item(&cache_key, &self.module_identifier, etag.clone());
    let Some(entry) = item_cache
      .get::<LoaderCacheEntry>()
      .map_err(|error| napi::Error::from_reason(error.to_string()))?
    else {
      return Ok(None);
    };
    let sidecar = if entry.requires_sidecar {
      let Some(sidecar) = self
        .cache
        .get_sidecar::<LoaderCacheSidecar>(&storage_key, &etag)
      else {
        return Ok(None);
      };
      Some(sidecar)
    } else {
      None
    };

    replay_dependency_delta(&mut input.file_dependencies, &entry.file_dependencies);
    replay_dependency_delta(&mut input.context_dependencies, &entry.context_dependencies);
    replay_dependency_delta(&mut input.missing_dependencies, &entry.missing_dependencies);
    replay_dependency_delta(&mut input.build_dependencies, &entry.build_dependencies);
    replay_parse_meta(&mut input.parse_meta, &entry.parse_meta);

    Ok(Some(JsLoaderCacheData {
      content: entry
        .content
        .clone()
        .map_or(Either::A(Null), |content| Either::B(content.into())),
      content_is_string: entry.content_is_string,
      source_map: entry.source_map.clone().map(Into::into),
      additional_data: sidecar.map(|sidecar| sidecar.additional_data.clone()),
      additional_data_cache_key: entry.additional_data_cache_key.clone(),
      file_dependencies: input.file_dependencies,
      context_dependencies: input.context_dependencies,
      missing_dependencies: input.missing_dependencies,
      build_dependencies: input.build_dependencies,
      parse_meta: input.parse_meta,
      cacheable: input.cacheable,
      has_unhandled_side_effects: false,
    }))
  }

  #[napi]
  pub fn store(
    &self,
    cache_key: String,
    input: JsLoaderCacheData,
    output: JsLoaderCacheData,
  ) -> napi::Result<Option<String>> {
    if !input.cacheable
      || input.has_unhandled_side_effects
      || !output.cacheable
      || output.has_unhandled_side_effects
    {
      return Ok(None);
    }
    let Some(etag) = input_etag(&input) else {
      return Ok(None);
    };
    let additional_data_cache_key = output_additional_data_cache_key(&cache_key, &etag, &output);
    let additional_data = output.additional_data.clone();
    let entry = LoaderCacheEntry {
      content: match &output.content {
        Either::A(_) => None,
        Either::B(content) => Some(content.to_vec()),
      },
      content_is_string: output.content_is_string,
      source_map: output
        .source_map
        .as_ref()
        .map(|source_map| source_map.to_vec()),
      additional_data_cache_key: additional_data_cache_key.clone(),
      file_dependencies: dependency_delta(&input.file_dependencies, &output.file_dependencies),
      context_dependencies: dependency_delta(
        &input.context_dependencies,
        &output.context_dependencies,
      ),
      missing_dependencies: dependency_delta(
        &input.missing_dependencies,
        &output.missing_dependencies,
      ),
      build_dependencies: dependency_delta(&input.build_dependencies, &output.build_dependencies),
      parse_meta: parse_meta_delta(&input.parse_meta, &output.parse_meta),
      requires_sidecar: additional_data.is_some(),
    };
    let (storage_key, item_cache) =
      self
        .cache
        .cache_item(&cache_key, &self.module_identifier, etag.clone());
    if let Some(additional_data) = additional_data {
      self
        .cache
        .store_sidecar(storage_key, etag, LoaderCacheSidecar { additional_data });
    } else {
      self.cache.remove_sidecar(&storage_key);
    }
    item_cache
      .store(CacheValue::new(entry))
      .map_err(|error| napi::Error::from_reason(error.to_string()))?;
    Ok(additional_data_cache_key)
  }
}
