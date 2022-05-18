use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use futures::lock::Mutex;
use napi::bindgen_prelude::*;
use napi::{
  threadsafe_function::{ErrorStrategy, ThreadSafeResultContext, ThreadsafeFunction},
  Env, JsObject, Result,
};
use napi_derive::napi;
use rspack_core::{BundleReactOptions, ResolveOption};
use serde::Deserialize;

pub mod adapter;
pub mod utils;

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
struct RawOptions {
  pub entries: Vec<String>,
  // pub format: InternalModuleFormat,
  pub minify: bool,
  pub root: Option<String>,
  pub outdir: Option<String>,
  pub entry_file_names: String, // | ((chunkInfo: PreRenderedChunk) => string)
  pub loader: Option<HashMap<String, String>>,
  pub inline_style: Option<bool>,
  pub alias: Option<HashMap<String, String>>,
  pub refresh: Option<bool>,
  pub source_map: Option<bool>,
  pub code_splitting: Option<bool>,
}

pub type Rspack = Arc<Mutex<RspackBundler>>;

#[napi(ts_return_type = "ExternalObject<RspackInternal>")]
#[allow(clippy::too_many_arguments)]
pub fn new_rspack(
  option_json: String,
  onload_callback: Option<JsFunction>,
  onresolve_callback: Option<JsFunction>,
) -> External<Rspack> {
  let options: RawOptions = serde_json::from_str(option_json.as_str()).unwrap();
  let loader = options.loader.map(|loader| parse_loader(loader));
  let onload_tsfn: Option<ThreadsafeFunction<String, ErrorStrategy::CalleeHandled>> =
    onload_callback.map(|x| {
      x.create_threadsafe_function(
        0,
        |ctx| ctx.env.create_string_from_std(ctx.value).map(|v| vec![v]),
        |ctx: ThreadSafeResultContext<Promise<String>>| {
          let return_value = ctx.return_value;

          ctx
            .env
            .execute_tokio_future(
              async move {
                let result = return_value.await?;

                let load_result: adapter::RspackThreadsafeResult<Option<adapter::OnLoadResult>> =
                  serde_json::from_str(&result).expect("failed to evaluate onload result");

                tracing::debug!("onload result {:?}", load_result);

                let sender = adapter::REGISTERED_ON_LOAD_SENDERS.remove(&load_result.get_call_id());

                if let Some((_, sender)) = sender {
                  sender.send(load_result.into_inner()).unwrap();
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
      .unwrap()
    });

  let onresolve_tsfn: Option<ThreadsafeFunction<String, ErrorStrategy::CalleeHandled>> =
    onresolve_callback.map(|x| {
      x.create_threadsafe_function(
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
                  sender.send(resolve_result.into_inner()).unwrap();
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
      .unwrap()
    });

  let rspack = RspackBundler::new(
    RspackBundlerOptions {
      entries: options.entries,
      minify: options.minify,
      code_splitting: options.code_splitting.unwrap_or_default(),
      outdir: options.outdir.unwrap_or_else(|| {
        std::env::current_dir()
          .unwrap()
          .join("./dist")
          .to_string_lossy()
          .to_string()
      }),
      source_map: options.source_map.unwrap_or_default(),
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
      root: options.root.unwrap_or_else(|| {
        std::env::current_dir()
          .unwrap()
          .to_string_lossy()
          .to_string()
      }),
      ..Default::default()
    },
    vec![Box::new(adapter::RspackPluginNodeAdapter {
      onload_tsfn,
      onresolve_tsfn,
    })],
  );
  create_external(Arc::new(Mutex::new(rspack)))
}

#[napi(ts_args_type = "rspack: ExternalObject<RspackInternal>")]
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
        Err(err) => Err(Error::new(Status::Unknown, err.to_string())),
      }
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

// for dts generation only
#[napi(object)]
struct RspackInternal {}
