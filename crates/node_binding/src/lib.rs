use std::sync::Arc;

use futures::lock::Mutex;
use napi::bindgen_prelude::*;
use napi::{Env, JsObject, Result};
use napi_derive::napi;
use serde::Deserialize;

use rspack::bundler::{
  BundleMode, BundleOptions as RspackBundlerOptions, Bundler as RspackBundler,
};

#[cfg(all(not(all(target_os = "linux", target_arch = "aarch64", target_env = "musl"))))]
#[global_allocator]
static ALLOC: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

fn create_external<T>(value: T) -> External<T> {
  External::new(value)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
struct BundleOptions {
  pub entries: Vec<String>,
  // pub format: InternalModuleFormat,
  pub minify: bool,
  pub outdir: Option<String>,
  pub entry_file_names: String, // | ((chunkInfo: PreRenderedChunk) => string)
}

struct RawRspack(RspackBundler);
type Rspack = Arc<Mutex<RawRspack>>;

#[napi]
fn new_rspack(option_json: String) -> External<Rspack> {
  let options: BundleOptions = serde_json::from_str(option_json.as_str()).unwrap();

  let raw_rspack = RawRspack(RspackBundler::new(
    RspackBundlerOptions {
      entries: options.entries,
      minify: options.minify,
      outdir: options.outdir,
      entry_file_names: options.entry_file_names,
      mode: BundleMode::Dev,
    },
    vec![],
  ));
  create_external(Arc::new(Mutex::new(raw_rspack)))
}

#[napi]
fn build(env: Env, rspack: External<Rspack>) -> Result<JsObject> {
  let bundler = (*rspack).clone();
  env.execute_tokio_future(
    async move {
      let mut bundler = bundler.lock().await;
      bundler.0.generate().await;
      bundler.0.write_assets_to_disk();
      Ok(0)
    },
    |env, ret| env.create_int32(ret),
  )
}
