use std::{
  sync::{Arc, OnceLock},
  time::{Duration, Instant},
};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{Cache, CompilerOptions, InfrastructureLogSink, create_cache};
use rspack_fs::{NativeFileSystem, ReadableFileSystem};

/// One shared cache, initialized from the first compiler that uses it.
#[napi]
pub struct JsCache {
  cache: OnceLock<Arc<Cache>>,
  build_start: Option<Instant>,
}

impl JsCache {
  pub fn get_or_initialize(
    &self,
    options: &CompilerOptions,
    input_filesystem: Arc<dyn ReadableFileSystem>,
    infrastructure_log_sink: Arc<dyn InfrastructureLogSink>,
  ) -> Arc<Cache> {
    Arc::clone(self.cache.get_or_init(|| {
      Arc::new(create_cache(
        options,
        input_filesystem,
        infrastructure_log_sink,
      ))
    }))
  }
}

#[napi]
impl JsCache {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      cache: OnceLock::new(),
      build_start: Some(Instant::now()),
    }
  }

  #[napi]
  pub fn begin_idle(&mut self) {
    let build_time = self
      .build_start
      .take()
      .map_or(Duration::ZERO, |start| start.elapsed());
    if let Some(cache) = self.cache.get() {
      cache.begin_idle(build_time);
    }
  }

  #[napi]
  pub fn end_idle(&mut self) {
    if let Some(cache) = self.cache.get() {
      cache.end_idle();
    }
    self.build_start = Some(Instant::now());
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
