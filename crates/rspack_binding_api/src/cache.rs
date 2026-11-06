use std::{
  sync::{Arc, OnceLock},
  time::{Duration, Instant},
};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{Cache, CacheOwner, CompilerOptions, InfrastructureLogSink, create_cache};
use rspack_fs::ReadableFileSystem;

/// One shared cache, initialized from the first compiler that uses it.
#[napi]
pub struct JsCache {
  owner: OnceLock<Arc<CacheOwner>>,
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
        .owner
        .get_or_init(|| create_cache(options, input_filesystem, infrastructure_log_sink))
        .cache(),
    )
  }
}

#[napi]
impl JsCache {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      owner: OnceLock::new(),
      build_start: Instant::now(),
    }
  }

  #[napi]
  pub fn begin_idle(&mut self) {
    let build_time = self.build_start.elapsed();
    if let Some(owner) = self.owner.get() {
      owner.cache().begin_idle(build_time);
    }
  }

  #[napi]
  pub fn end_idle(&mut self) {
    if let Some(owner) = self.owner.get() {
      owner.cache().end_idle();
    }
    self.build_start = Instant::now();
  }

  #[napi(ts_return_type = "Promise<void>")]
  pub fn shutdown<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    let owner = self.owner.take();
    rspack_napi::runtime::promise_from_future(env, async move {
      if let Some(owner) = owner {
        owner.shutdown().await;
      }
      Ok(())
    })
  }
}
