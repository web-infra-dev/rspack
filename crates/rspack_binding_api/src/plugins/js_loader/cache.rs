use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_cacheable::cacheable;
use rspack_core::{
  CacheFacade, CacheValue, Content, Etag, LoaderCacheDependencySnapshot, LoaderDependencies,
  Resolver, loader_cache_dependency_snapshot, loader_cache_dependency_snapshot_is_valid,
  loader_cache_etag, loader_cache_item, restore_loader_cache_dependencies,
};
use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_hash::{HashFunction, RspackHasher};
use rspack_loader_runner::LoaderRunnerOptions;
use rspack_paths::Utf8Path;

use super::context::JsLoaderDependencies;

#[cacheable]
#[derive(Clone)]
struct LoaderCacheEntry {
  content: Option<Vec<u8>>,
  content_is_string: bool,
  source_map: Option<Vec<u8>>,
  dependency_snapshot: LoaderCacheDependencySnapshot,
}

#[napi(object)]
pub struct JsLoaderCacheEntry {
  pub content: Either3<Null, String, Uint8Array>,
  pub source_map: Option<Uint8Array>,
  pub added_dependencies: JsLoaderDependencies,
  pub removed_dependencies: JsLoaderDependencies,
}

#[napi]
pub struct JsLoaderCache {
  cache: CacheFacade,
  fs: Arc<dyn ReadableFileSystem>,
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
      fs: instance.fs.clone(),
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
  fn new(
    cache: CacheFacade,
    fs: Arc<dyn ReadableFileSystem>,
    module_identifier: String,
    loaders: Vec<LoaderRunnerOptions>,
  ) -> Self {
    let pending_etags = Arc::new(Mutex::new(vec![None; loaders.len()]));
    Self {
      cache,
      fs,
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
    fs: Arc<dyn ReadableFileSystem>,
    module_identifier: String,
    loaders: Vec<LoaderRunnerOptions>,
  ) -> Self {
    Self(JsLoaderCache::new(cache, fs, module_identifier, loaders))
  }
}

pub(crate) async fn loader_cache_version(
  resolver: &Resolver,
  path: &Utf8Path,
) -> Result<Option<String>> {
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
    content: Either<String, Uint8Array>,
  ) -> napi::Result<Option<JsLoaderCacheEntry>> {
    let loader = self.loader(loader_index)?;
    let content = match content {
      Either::A(content) => content.into_bytes(),
      Either::B(content) => content.to_vec(),
    };
    let etag = loader_cache_etag(
      &Content::Buffer(content),
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
    if !loader_cache_dependency_snapshot_is_valid(self.fs.as_ref(), &entry.dependency_snapshot) {
      self.set_pending_etag(loader_index, Some(etag))?;
      return Ok(None);
    }
    self.set_pending_etag(loader_index, None)?;
    let mut dependencies = LoaderDependencies::default();
    restore_loader_cache_dependencies(&entry.dependency_snapshot, &mut dependencies);

    Ok(Some(JsLoaderCacheEntry {
      content: match (&entry.content, entry.content_is_string) {
        (None, _) => Either3::A(Null),
        (Some(content), true) => {
          Either3::B(String::from_utf8(content.clone()).map_err(|error| {
            napi::Error::from_reason(format!("Invalid UTF-8 in loader cache entry: {error}"))
          })?)
        }
        (Some(content), false) => Either3::C(content.clone().into()),
      },
      source_map: entry.source_map.clone().map(Into::into),
      added_dependencies: (&dependencies).into(),
      removed_dependencies: Default::default(),
    }))
  }

  #[napi]
  pub fn store(&self, loader_index: u32, output: JsLoaderCacheEntry) -> napi::Result<()> {
    let loader_name = &self.loader(loader_index)?.loader_name;
    let Some(etag) = self.take_pending_etag(loader_index)? else {
      return Ok(());
    };
    if !output.removed_dependencies.is_empty()
      || !output.added_dependencies.context_dependencies.is_empty()
      || !output.added_dependencies.missing_dependencies.is_empty()
    {
      return Ok(());
    }
    let dependencies: LoaderDependencies = output.added_dependencies.into();
    let Some(dependency_snapshot) =
      loader_cache_dependency_snapshot(self.fs.as_ref(), &dependencies)
    else {
      return Ok(());
    };
    let (content, content_is_string) = match output.content {
      Either3::A(_) => (None, false),
      Either3::B(content) => (Some(content.into_bytes()), true),
      Either3::C(content) => (Some(content.to_vec()), false),
    };
    let entry = LoaderCacheEntry {
      content,
      content_is_string,
      source_map: output.source_map.map(|source_map| source_map.to_vec()),
      dependency_snapshot,
    };
    let item_cache = loader_cache_item(&self.cache, &self.module_identifier, loader_name, etag);
    item_cache
      .store(CacheValue::new(entry))
      .map_err(|error| napi::Error::from_reason(error.to_string()))
  }
}
