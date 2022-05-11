use std::collections::HashMap;
use std::sync::Arc;

use futures::lock::Mutex;
use napi::bindgen_prelude::*;
use napi::{Env, JsObject, Result};
use napi_derive::napi;
use rspack_core::{BundleReactOptions, ResolveOption};
use serde::Deserialize;

use rspack::bundler::{
  BundleMode, BundleOptions as RspackBundlerOptions, Bundler as RspackBundler,
};

#[cfg(all(not(all(target_os = "linux", target_arch = "aarch64", target_env = "musl"))))]
#[global_allocator]
static ALLOC: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

pub fn create_external<T>(value: T) -> External<T> {
  External::new(value)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
struct BundleOptions {
  pub entries: Vec<String>,
  // pub format: InternalModuleFormat,
  pub minify: bool,
  pub outdir: Option<String>,
  pub entry_file_names: String, // | ((chunkInfo: PreRenderedChunk) => string)
  pub loader: Option<HashMap<String, String>>,
  pub inline_style: Option<bool>,
  pub alias: Option<HashMap<String, String>>,
  pub refresh: Option<bool>,
}

pub type Rspack = Arc<Mutex<RspackBundler>>;

#[napi]
pub fn new_rspack(option_json: String) -> External<Rspack> {
  let options: BundleOptions = serde_json::from_str(option_json.as_str()).unwrap();
  let loader = options.loader.map(|loader| parse_loader(loader));
  let rspack = RspackBundler::new(
    RspackBundlerOptions {
      entries: options.entries,
      minify: options.minify,
      outdir: options.outdir.unwrap_or_else(|| {
        std::env::current_dir()
          .unwrap()
          .join("./dist")
          .to_string_lossy()
          .to_string()
      }),
      entry_file_names: options.entry_file_names,
      mode: BundleMode::Dev,
      loader,
      inline_style: options.inline_style.unwrap_or_default(),
      react: BundleReactOptions {
        refresh: options.refresh.unwrap_or_default(),
        ..Default::default()
      },
      resolve: ResolveOption {
        alias: options
          .alias
          .unwrap_or_default()
          .into_iter()
          .map(|(s1, s2)| (s1, Some(s2)))
          .collect::<Vec<_>>(),
        ..Default::default()
      },
      ..Default::default()
    },
    vec![],
  );
  create_external(Arc::new(Mutex::new(rspack)))
}

#[napi]
pub fn build(env: Env, rspack: External<Rspack>) -> Result<JsObject> {
  let bundler = (*rspack).clone();
  env.execute_tokio_future(
    async move {
      let mut bundler = bundler.lock().await;
      bundler.build().await;
      bundler.write_assets_to_disk();
      Ok(())
    },
    |_env, ret| Ok(ret),
  )
}

#[napi]
pub fn rebuild(env: Env, rspack: External<Rspack>, chnaged_file: String) -> Result<JsObject> {
  let bundler = (*rspack).clone();
  env.execute_tokio_future(
    async move {
      let mut bundler = bundler.lock().await;
      let changed = bundler.rebuild(chnaged_file).await;
      bundler.write_assets_to_disk();
      Ok(changed)
    },
    |_env, ret| Ok(ret),
  )
}

fn parse_loader(user_input: HashMap<String, String>) -> rspack_core::LoaderOptions {
  user_input
    .into_iter()
    .filter_map(|(ext, loader)| {
      let loader = match loader.as_str() {
        "dataURI" => Some(rspack_core::Loader::DataURI),
        "json" => Some(rspack_core::Loader::Json),
        "text" => Some(rspack_core::Loader::Text),
        _ => None,
      }?;
      Some((ext, loader))
    })
    .collect()
}
