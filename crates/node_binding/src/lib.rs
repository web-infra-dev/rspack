use futures::prelude::*;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
use napi::sys::napi_env;
use napi::{CallContext, JsNumber, JsObject, Result};
use napi_derive::{js_function, module_exports, napi};
use serde::Deserialize;

use rspack::bundler::{BundleOptions as RspackBundlerOptions, Bundler as RspackBundler};

#[cfg(all(not(all(target_os = "linux", target_arch = "aarch64", target_env = "musl"))))]
#[global_allocator]
static ALLOC: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

fn create_external<T>(value: T) -> External<T> {
  External::new(value)
}

#[derive(Deserialize)]
#[napi(object)]
struct BundleOptions {
  pub entries: Vec<String>,
  // pub format: InternalModuleFormat,
  pub minify: bool,
  pub outdir: Option<String>,
  pub entry_file_names: String, // | ((chunkInfo: PreRenderedChunk) => string)
}

pub struct Rspack(RspackBundler);

#[napi]
fn new_rspack(option_json: String) -> External<Rspack> {
  let options: BundleOptions = serde_json::from_str(option_json.as_str()).unwrap();

  let raw_rspack = Rspack(RspackBundler::new(
    RspackBundlerOptions {
      entries: options.entries,
      minify: options.minify,
      outdir: options.outdir,
      entry_file_names: options.entry_file_names,
    },
    vec![],
  ));
  create_external(raw_rspack)
}

#[js_function(1)]
pub fn build(ctx: CallContext) -> Result<JsObject> {
  // ctx.get(0);
  ctx.env.execute_tokio_future(
    async move {
      let ret = 123;
      Ok(ret)
    },
    |env, ret| env.create_int32(ret),
  )
}

#[module_exports]
pub fn init(mut exports: JsObject) -> Result<()> {
  exports.create_named_method("build", build)?;
  Ok(())
}
