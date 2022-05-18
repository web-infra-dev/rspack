use std::fmt::Debug;
use std::sync::{
  atomic::{AtomicUsize, Ordering},
  Arc,
};

use rspack_core::{BundleContext, Plugin, PluginLoadHookOutput, PluginResolveHookOutput};

use async_trait::async_trait;
use dashmap::DashMap;
use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot::{self, Sender};

pub static CALL_ID: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(1));

pub static REGISTERED_ON_LOAD_SENDERS: Lazy<Arc<DashMap<usize, Sender<Option<OnLoadResult>>>>> =
  Lazy::new(|| Default::default());

pub static REGISTERED_ON_RESOLVE_SENDERS: Lazy<
  Arc<DashMap<usize, Sender<Option<OnResolveResult>>>>,
> = Lazy::new(|| Default::default());

pub struct RspackPluginNodeAdapter {
  pub onload_tsfn: Option<ThreadsafeFunction<String, ErrorStrategy::CalleeHandled>>,
  pub onresolve_tsfn: Option<ThreadsafeFunction<String, ErrorStrategy::CalleeHandled>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RspackThreadsafeContext<T: Debug> {
  call_id: usize,
  inner: T,
}

impl<T: Debug> RspackThreadsafeContext<T> {
  pub fn new(payload: T) -> Self {
    Self {
      call_id: CALL_ID.fetch_add(1, Ordering::SeqCst),
      inner: payload,
    }
  }

  pub fn into_inner(self) -> T {
    self.inner
  }

  #[inline]
  pub fn get_call_id(&self) -> usize {
    self.call_id
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RspackThreadsafeResult<T: Debug> {
  call_id: usize,
  inner: T,
}

impl<T: Debug> RspackThreadsafeResult<T> {
  pub fn into_inner(self) -> T {
    self.inner
  }

  #[inline]
  pub fn get_call_id(&self) -> usize {
    self.call_id
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[napi(object)]
pub struct OnLoadContext {
  pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct OnLoadResult {
  pub content: Option<String>,
  pub loader: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[napi(object)]
pub struct OnResolveContext {
  pub importer: Option<String>,
  pub importee: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct OnResolveResult {
  pub uri: String,
  pub external: bool,
}

impl Debug for RspackPluginNodeAdapter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RspackPluginNodeAdapter").finish()
  }
}

#[async_trait]
impl Plugin for RspackPluginNodeAdapter {
  fn name(&self) -> &'static str {
    "rspack_plugin_node_adapter"
  }

  #[tracing::instrument(skip_all)]
  async fn load(&self, _ctx: &BundleContext, id: &str) -> PluginLoadHookOutput {
    let load_context = RspackThreadsafeContext::new(OnLoadContext { id: id.to_owned() });

    let (tx, rx) = oneshot::channel::<Option<OnLoadResult>>();

    match REGISTERED_ON_LOAD_SENDERS.entry(load_context.call_id) {
      dashmap::mapref::entry::Entry::Occupied(_) => {
        panic!(
          "duplicated call id encountered {}, please file an issue.",
          load_context.call_id
        );
      }
      dashmap::mapref::entry::Entry::Vacant(v) => {
        v.insert(tx);
      }
    }

    let serialized_load_context = serde_json::to_string(&load_context).unwrap();
    if let Some(onload_tsfn) = &self.onload_tsfn {
      onload_tsfn.call(
        Ok(serialized_load_context),
        ThreadsafeFunctionCallMode::Blocking,
      );
    }

    let load_result = rx.await.expect("failed to receive onload result");

    tracing::debug!("[rspack:binding] load result {:#?}", load_result);

    load_result.map(|result| rspack_core::LoadedSource {
      loader: result.loader.map(|loader| {
        use rspack_core::Loader;

        match loader.as_str() {
          "data_uri" => Loader::DataURI,
          "json" => Loader::Json,
          "text" => Loader::Text,
          "css" => Loader::Css,
          "js" => Loader::Js,
          "jsx" => Loader::Jsx,
          "ts" => Loader::Ts,
          "tsx" => Loader::Tsx,
          "null" => Loader::Null,
          _ => panic!("unexpected loader option `{}`", loader),
        }
      }),
      content: result.content,
    })
  }

  #[tracing::instrument(skip_all)]
  async fn resolve(
    &self,
    _ctx: &BundleContext,
    importee: &str,
    importer: Option<&str>,
  ) -> PluginResolveHookOutput {
    let resolve_context = RspackThreadsafeContext::new(OnResolveContext {
      importer: importer.map(|s| s.to_owned()),
      importee: importee.to_owned(),
    });

    let (tx, rx) = oneshot::channel::<Option<OnResolveResult>>();

    match REGISTERED_ON_RESOLVE_SENDERS.entry(resolve_context.call_id) {
      dashmap::mapref::entry::Entry::Occupied(_) => {
        panic!(
          "duplicated call id encountered {}, please file an issue.",
          resolve_context.call_id
        );
      }
      dashmap::mapref::entry::Entry::Vacant(v) => {
        v.insert(tx);
      }
    }

    let serialized_resolve_context = serde_json::to_string(&resolve_context).unwrap();
    if let Some(onresolve_tsfn) = &self.onresolve_tsfn {
      onresolve_tsfn.call(
        Ok(serialized_resolve_context),
        ThreadsafeFunctionCallMode::Blocking,
      );
    }

    let resolve_result = rx.await.expect("failed to receive onresolve result");

    tracing::debug!("[rspack:binding] resolve result {:#?}", resolve_result);

    resolve_result.map(|result| rspack_core::ResolvedURI {
      uri: result.uri,
      external: result.external,
    })
  }
}
