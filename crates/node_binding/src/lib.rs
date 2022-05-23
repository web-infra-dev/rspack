use std::path::Path;
use std::sync::Arc;

use futures::lock::Mutex;
use napi::bindgen_prelude::*;
use napi::{
  threadsafe_function::{ErrorStrategy, ThreadSafeResultContext, ThreadsafeFunction},
  Env, JsObject, Result,
};
use napi_derive::napi;
use nodejs_resolver::{ResolveResult, Resolver, ResolverOptions};

pub mod adapter;
mod options;
pub mod utils;
pub use options::*;

use rspack::bundler::Bundler as RspackBundler;

#[cfg(all(not(all(target_os = "linux", target_arch = "aarch64", target_env = "musl"))))]
#[global_allocator]
static ALLOC: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

pub fn create_external<T>(value: T) -> External<T> {
  External::new(value)
}

pub type Rspack = Arc<Mutex<RspackBundler>>;

#[napi(object)]
pub struct PluginCallbacks {
  pub onload_callback: JsFunction,
  pub onresolve_callback: JsFunction,
}

#[napi(ts_return_type = "ExternalObject<RspackInternal>")]
#[allow(clippy::too_many_arguments)]
pub fn new_rspack(
  env: Env,
  option_json: String,
  plugin_callbacks: Option<PluginCallbacks>,
) -> External<Rspack> {
  let options: RawOptions = serde_json::from_str(option_json.as_str()).unwrap();

  let node_adapter = plugin_callbacks.map(
    |PluginCallbacks {
       onload_callback,
       onresolve_callback,
     }| {
      let mut onload_tsfn: ThreadsafeFunction<String, ErrorStrategy::CalleeHandled> =
        onload_callback
          .create_threadsafe_function(
            0,
            |ctx| ctx.env.create_string_from_std(ctx.value).map(|v| vec![v]),
            |ctx: ThreadSafeResultContext<Promise<String>>| {
              let return_value = ctx.return_value;

              ctx
                .env
                .execute_tokio_future(
                  async move {
                    let result = return_value.await?;

                    let load_result: adapter::RspackThreadsafeResult<
                      Option<adapter::OnLoadResult>,
                    > = serde_json::from_str(&result).expect("failed to evaluate onload result");

                    tracing::debug!("onload result {:?}", load_result);

                    let sender =
                      adapter::REGISTERED_ON_LOAD_SENDERS.remove(&load_result.get_call_id());

                    if let Some((_, sender)) = sender {
                      sender
                        .send(load_result.into_inner())
                        .expect("unable to send");
                    } else {
                      panic!("unable to send");
                    }

                    Ok(())
                  },
                  |_, ret| Ok(ret),
                )
                .expect("failed to execute tokio future");
            },
          )
          .unwrap();

      let mut onresolve_tsfn: ThreadsafeFunction<String, ErrorStrategy::CalleeHandled> =
        onresolve_callback
          .create_threadsafe_function(
            0,
            |ctx| ctx.env.create_string_from_std(ctx.value).map(|v| vec![v]),
            |ctx: ThreadSafeResultContext<Promise<String>>| {
              let return_value = ctx.return_value;

              ctx
                .env
                .execute_tokio_future(
                  async move {
                    let result = return_value.await?;

                    let resolve_result: adapter::RspackThreadsafeResult<
                      Option<adapter::OnResolveResult>,
                    > = serde_json::from_str(&result).expect("failed to evaluate onresolve result");

                    tracing::debug!("[rspack:binding] onresolve result {:?}", resolve_result);

                    let sender =
                      adapter::REGISTERED_ON_RESOLVE_SENDERS.remove(&resolve_result.get_call_id());

                    if let Some((_, sender)) = sender {
                      sender
                        .send(resolve_result.into_inner())
                        .expect("unable to send");
                    } else {
                      panic!("unable to send");
                    }

                    Ok(())
                  },
                  |_, ret| Ok(ret),
                )
                .expect("failed to execute tokio future");
            },
          )
          .unwrap();

      onload_tsfn.unref(&env).unwrap();
      onresolve_tsfn.unref(&env).unwrap();

      adapter::RspackPluginNodeAdapter {
        onload_tsfn,
        onresolve_tsfn,
      }
    },
  );

  let mut plugins = vec![];

  if let Some(node_adapter) = node_adapter {
    plugins.push(Box::new(node_adapter) as Box<dyn rspack_core::Plugin>);
  }

  let rspack = RspackBundler::new(normalize_bundle_options(options), plugins);
  create_external(Arc::new(Mutex::new(rspack)))
}

#[napi(ts_args_type = "rspack: ExternalObject<RspackInternal>")]
pub fn build(env: Env, rspack: External<Rspack>) -> Result<JsObject> {
  let bundler = (*rspack).clone();
  env.execute_tokio_future(
    async move {
      let mut bundler = bundler.lock().await;
      let map = bundler.build(None).await;
      bundler.write_assets_to_disk();
      Ok(map)
    },
    |_env, ret| Ok(ret),
  )
}

#[napi(ts_args_type = "rspack: ExternalObject<RspackInternal>, changedFile: string")]
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
#[napi(object)]
struct ResolveRet {
  pub status: bool,
  pub result: Option<String>,
}
#[napi(ts_args_type = "rspack: ExternalObject<RspackInternal>, id: string, dir: string")]
pub fn resolve(env: Env, rspack: External<Rspack>, id: String, dir: String) -> Result<JsObject> {
  let bundler = (*rspack).clone();
  env.execute_tokio_future(
    async move {
      let mut bundler = bundler.lock().await;
      let res = bundler.resolve(id, dir);
      match res {
        Ok(val) => {
          if let nodejs_resolver::ResolveResult::Path(xx) = val {
            Ok(ResolveRet {
              status: true,
              result: Some(xx.to_string_lossy().to_string()),
            })
          } else {
            Ok(ResolveRet {
              status: false,
              result: None,
            })
          }
        }
        Err(err) => Err(Error::new(Status::Unknown, err)),
      }
    },
    |_env, ret| Ok(ret),
  )
}

#[napi]
pub fn resolve_file(base_dir: String, import_path: String) -> Result<String> {
  let resolver = Resolver::new(ResolverOptions {
    extensions: vec!["less", "css", "scss", "sass", "js"]
      .into_iter()
      .map(|s| s.to_owned())
      .collect(),
    ..Default::default()
  });
  match resolver.resolve(Path::new(&base_dir), &import_path) {
    Ok(res) => {
      if let ResolveResult::Path(abs_path) = res {
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
struct RspackInternal {}
