use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use futures::lock::Mutex;
use napi::{bindgen_prelude::*, JsString};
use napi::{
  threadsafe_function::{
    ErrorStrategy, ThreadSafeResultContext, ThreadsafeFunction, ThreadsafeFunctionCallMode,
  },
  Env, JsObject, Result,
};
use napi_derive::napi;
use once_cell::sync::Lazy;
use rspack_core::{BundleContext, BundleReactOptions, Plugin, PluginLoadHookOutput, ResolveOption};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot::{self, Sender};

use rspack::bundler::{
  BundleMode, BundleOptions as RspackBundlerOptions, Bundler as RspackBundler,
};
pub mod utils;

#[cfg(all(not(all(target_os = "linux", target_arch = "aarch64", target_env = "musl"))))]
#[global_allocator]
static ALLOC: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

static CALL_ID: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(1));

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

static RESULT_LIST: Lazy<Arc<DashMap<usize, Sender<OnLoadContext>>>> =
  Lazy::new(|| Default::default());

struct RspackPluginNodeAdapter {
  onload_tsfn: ThreadsafeFunction<String, ErrorStrategy::CalleeHandled>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RspackThreadsafeContext<T: Debug> {
  call_id: usize,
  payload: T,
}

#[derive(Serialize, Deserialize, Debug)]
struct OnLoadContext {
  id: String,
}

impl RspackPluginNodeAdapter {}

impl Debug for RspackPluginNodeAdapter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RspackPluginNodeAdapter").finish()
  }
}

fn wrap_rspack_tsfn_context<T: Debug>(payload: T) -> RspackThreadsafeContext<T> {
  let current_call_id = CALL_ID.fetch_add(1, Ordering::SeqCst);

  RspackThreadsafeContext {
    call_id: current_call_id,
    payload,
  }
}

#[async_trait]
impl Plugin for RspackPluginNodeAdapter {
  fn name(&self) -> &'static str {
    "rspack_plugin_node_adapter"
  }

  async fn load(&self, _ctx: &BundleContext, id: &str) -> PluginLoadHookOutput {
    let load_ctxt = wrap_rspack_tsfn_context(OnLoadContext { id: id.to_owned() });

    let (tx, rx) = oneshot::channel::<OnLoadContext>();

    match RESULT_LIST.entry(load_ctxt.call_id) {
      dashmap::mapref::entry::Entry::Occupied(_) => {}
      dashmap::mapref::entry::Entry::Vacant(v) => {
        v.insert(tx);
      }
    }

    let serialized_load_ctxt = serde_json::to_string(&load_ctxt).unwrap();

    self.onload_tsfn.call(
      Ok(serialized_load_ctxt),
      ThreadsafeFunctionCallMode::NonBlocking,
    );

    let load_result = rx.await;

    println!("load result from node {:#?}", load_result);

    None
  }
}

#[napi(ts_return_type = "ExternalObject<RspackInternal>")]
pub fn new_rspack(option_json: String, onload_callback: JsFunction) -> External<Rspack> {
  let options: RawOptions = serde_json::from_str(option_json.as_str()).unwrap();
  let loader = options.loader.map(|loader| parse_loader(loader));

  let onload_tsfn: ThreadsafeFunction<String, ErrorStrategy::CalleeHandled> = onload_callback
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

              let load_context: RspackThreadsafeContext<OnLoadContext> =
                serde_json::from_str(result.as_str()).unwrap();

              let sender = RESULT_LIST.remove(&load_context.call_id);

              if let Some(sender) = sender {
                sender.1.send(load_context.payload).unwrap();
              } else {
                print!("unable to send");
              }

              Ok(())
            },
            |_, ret| Ok(ret),
          )
          .expect("failed to execute tokio future");
      },
    )
    .unwrap();

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
    vec![Box::new(RspackPluginNodeAdapter {
      onload_tsfn: onload_tsfn.clone(),
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
