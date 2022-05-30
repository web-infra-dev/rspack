#![deny(clippy::all)]

use std::path::Path;
use std::sync::Arc;

use futures::lock::Mutex;
use napi::bindgen_prelude::*;
use napi::{Env, JsObject, Result};
use napi_derive::napi;
use nodejs_resolver::Resolver;

pub mod adapter;
mod options;
pub mod utils;

use adapter::utils::create_node_adapter_from_plugin_callbacks;
pub use options::*;

use rspack::bundler::Bundler as RspackBundler;
use rspack::stats::Stats;

#[cfg(all(not(all(target_os = "linux", target_arch = "aarch64", target_env = "musl"))))]
#[global_allocator]
static ALLOC: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

pub fn create_external<T>(value: T) -> External<T> {
  External::new(value)
}

pub type Rspack = Arc<Mutex<RspackBundler>>;

#[napi(object)]
pub struct PluginCallbacks {
  pub build_start_callback: JsFunction,
  pub load_callback: JsFunction,
  pub resolve_callback: JsFunction,
  pub build_end_callback: JsFunction,
}

pub struct RspackBindingContext {
  pub rspack: Rspack,
  pub resolver: Arc<Resolver>,
}

#[napi(ts_return_type = "ExternalObject<RspackInternal>")]
#[allow(clippy::too_many_arguments)]
pub fn new_rspack(
  env: Env,
  option_json: String,
  plugin_callbacks: Option<PluginCallbacks>,
) -> Result<External<RspackBindingContext>> {
  let options: RawOptions = serde_json::from_str(option_json.as_str()).unwrap();

  let node_adapter = create_node_adapter_from_plugin_callbacks(&env, plugin_callbacks);

  let mut plugins = vec![];

  if let Some(node_adapter) = node_adapter {
    plugins.push(Box::new(node_adapter) as Box<dyn rspack_core::Plugin>);
  }

  let rspack = RspackBundler::new(normalize_bundle_options(options)?, plugins);

  let resolver = rspack.resolver.clone();

  Ok(create_external(RspackBindingContext {
    rspack: Arc::new(Mutex::new(rspack)),
    resolver,
  }))
}

#[napi(
  ts_args_type = "rspack: ExternalObject<RspackInternal>",
  ts_return_type = "Promise<Record<string, string>>"
)]
pub fn build(env: Env, binding_context: External<RspackBindingContext>) -> Result<JsObject> {
  let bundler = (*binding_context).rspack.clone();
  env.execute_tokio_future(
    async move {
      let mut bundler = bundler.lock().await;
      let Stats { map, .. } = bundler
        .build(None)
        .await
        .map_err(|e| Error::new(napi::Status::Unknown, e.to_string()))?;
      bundler.write_assets_to_disk();
      Ok(map)
    },
    |_env, ret| Ok(ret),
  )
}

#[napi(
  ts_args_type = "rspack: ExternalObject<RspackInternal>, changedFile: string[]",
  ts_return_type = "Promise<[diff: Record<string, string>, map: Record<string, string>]>"
)]
pub fn rebuild(
  env: Env,
  binding_context: External<RspackBindingContext>,
  changed_file: Vec<String>,
) -> Result<JsObject> {
  let bundler = (*binding_context).rspack.clone();
  env.execute_tokio_future(
    async move {
      let mut bundler = bundler.lock().await;
      let changed = bundler
        .rebuild(changed_file)
        .await
        .map_err(|e| Error::new(napi::Status::Unknown, e.to_string()))?;
      bundler.write_assets_to_disk();
      Ok(changed)
    },
    |_env, ret| Ok(ret),
  )
}

#[napi(
  ts_args_type = "rspack: ExternalObject<RspackInternal>, source: string, resolveOptions: ResolveOptions",
  ts_return_type = "ResolveResult"
)]
pub fn resolve(
  binding_context: External<RspackBindingContext>,
  source: String,
  resolve_options: ResolveOptions,
) -> Result<ResolveResult> {
  let resolver = (*binding_context).resolver.clone();
  println!("[rust] resolve: {}", source);
  let res = resolver.resolve(Path::new(&resolve_options.resolve_dir), &source);
  println!("[rust] resolve result {:#?}", res);
  match res {
    Ok(val) => {
      if let nodejs_resolver::ResolveResult::Path(p) = val {
        Ok(ResolveResult {
          status: true,
          path: Some(p.to_string_lossy().to_string()),
        })
      } else {
        Ok(ResolveResult {
          status: false,
          path: None,
        })
      }
    }
    Err(err) => Err(Error::new(Status::Unknown, err)),
  }
}

#[napi(object)]
pub struct ResolveOptions {
  pub resolve_dir: String,
}

#[napi(object)]
pub struct ResolveResult {
  pub status: bool,
  pub path: Option<String>,
}

#[napi]
pub fn resolve_file(base_dir: String, import_path: String) -> Result<String> {
  let resolver = Resolver::new(nodejs_resolver::ResolverOptions {
    extensions: vec!["less", "css", "scss", "sass", "js"]
      .into_iter()
      .map(|s| s.to_owned())
      .collect(),
    ..Default::default()
  });
  match resolver.resolve(Path::new(&base_dir), &import_path) {
    Ok(res) => {
      if let nodejs_resolver::ResolveResult::Path(abs_path) = res {
        Ok(abs_path.to_str().unwrap().to_string())
      } else {
        Ok(import_path)
      }
    }
    Err(msg) => Err(Error::new(Status::Unknown, msg)),
  }
}

// for dts generation only
#[napi(object)]
pub struct RspackInternal {}
