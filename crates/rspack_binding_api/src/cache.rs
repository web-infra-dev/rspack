use std::{
  sync::{Arc, LazyLock, OnceLock, Weak},
  time::{Duration, Instant},
};

use dashmap::{DashMap, mapref::entry::Entry};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{Cache, CompilerOptions, InfrastructureLogSink, create_cache};
use rspack_fs::{NativeFileSystem, ReadableFileSystem};
use rspack_paths::Utf8PathBuf;

type StorageDir = Utf8PathBuf;

static SHARED_PERSISTENT_CACHES: LazyLock<DashMap<StorageDir, Weak<Cache>>> =
  LazyLock::new(DashMap::new);

fn get_or_create_cache(
  options: &CompilerOptions,
  input_filesystem: Arc<dyn ReadableFileSystem>,
  infrastructure_log_sink: Arc<dyn InfrastructureLogSink>,
) -> Arc<Cache> {
  let storage_dir = match (&options.experiments.new_cache.is_enabled(), &options.cache) {
    (true, rspack_core::CacheOptions::Persistent(options)) => match &options.storage {
      rspack_core::StorageOptions::FileSystem { location, .. } => Some(location.clone()),
    },
    _ => None,
  };

  let Some(storage_dir) = storage_dir else {
    return Arc::new(create_cache(
      options,
      input_filesystem,
      infrastructure_log_sink,
    ));
  };

  match SHARED_PERSISTENT_CACHES.entry(storage_dir) {
    Entry::Occupied(mut entry) => {
      if let Some(cache) = entry.get().upgrade()
        && !cache.is_closing()
      {
        return cache;
      }
      let cache = Arc::new(create_cache(
        options,
        input_filesystem,
        infrastructure_log_sink,
      ));
      entry.insert(Arc::downgrade(&cache));
      cache
    }
    Entry::Vacant(entry) => {
      let cache = Arc::new(create_cache(
        options,
        input_filesystem,
        infrastructure_log_sink,
      ));
      entry.insert(Arc::downgrade(&cache));
      cache
    }
  }
}

/// One shared cache, initialized from the first compiler that uses it.
#[napi]
pub struct JsCache {
  cache: OnceLock<Arc<Cache>>,
  build_start: Instant,
}

impl JsCache {
  pub fn get_or_initialize(
    &self,
    options: &CompilerOptions,
    input_filesystem: Arc<dyn ReadableFileSystem>,
    infrastructure_log_sink: Arc<dyn InfrastructureLogSink>,
  ) -> Arc<Cache> {
    Arc::clone(
      self
        .cache
        .get_or_init(|| get_or_create_cache(options, input_filesystem, infrastructure_log_sink)),
    )
  }
}

#[napi]
impl JsCache {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      cache: OnceLock::new(),
      build_start: Instant::now(),
    }
  }

  #[napi]
  pub fn begin_idle(&mut self) {
    let build_time = self.build_start.elapsed();
    if let Some(cache) = self.cache.get() {
      cache.begin_idle(build_time);
    }
  }

  #[napi]
  pub fn end_idle(&mut self) {
    if let Some(cache) = self.cache.get() {
      cache.end_idle();
    }
    self.build_start = Instant::now();
  }

  #[napi(ts_return_type = "Promise<void>")]
  pub fn shutdown<'env>(&self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    // Own storage until shutdown completes, independently of the JS wrapper.
    let cache = self.cache.get().cloned();
    rspack_napi::runtime::promise_from_future(env, async move {
      if let Some(cache) = cache {
        cache.shutdown().await;
      }
      Ok(())
    })
  }
}
