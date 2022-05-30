use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};

use napi::Error;
use rspack::ast::Str;
use rspack_core::{
  BundleContext, LoadArgs, Plugin, PluginLoadHookOutput, PluginResolveHookOutput, ResolveArgs,
};

use async_trait::async_trait;
use napi::threadsafe_function::ThreadsafeFunctionCallMode;
use napi_derive::napi;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

mod common;
pub mod utils;

use common::ThreadsafeRspackCallback;
use common::{
  REGISTERED_BUILD_END_SENDERS, REGISTERED_BUILD_START_SENDERS, REGISTERED_LOAD_SENDERS,
  REGISTERED_RESOLVE_SENDERS,
};

pub static CALL_ID: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(1));

pub struct RspackPluginNodeAdapter {
  pub build_start_tsfn: ThreadsafeRspackCallback,
  pub load_tsfn: ThreadsafeRspackCallback,
  pub resolve_tsfn: ThreadsafeRspackCallback,
  pub build_end_tsfn: ThreadsafeRspackCallback,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RspackThreadsafeContext<T: Debug> {
  id: usize,
  inner: T,
}

impl<T: Debug> RspackThreadsafeContext<T> {
  pub fn new(payload: T) -> Self {
    Self {
      id: CALL_ID.fetch_add(1, Ordering::SeqCst),
      inner: payload,
    }
  }

  pub fn into_inner(self) -> T {
    self.inner
  }

  #[inline(always)]
  pub fn get_call_id(&self) -> usize {
    self.id
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RspackThreadsafeResult<T: Debug> {
  id: usize,
  inner: T,
}

impl<T: Debug> RspackThreadsafeResult<T> {
  pub fn into_inner(self) -> T {
    self.inner
  }

  #[inline(always)]
  pub fn get_call_id(&self) -> usize {
    self.id
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[napi(object)]
pub struct OnLoadContext {
  pub id: String,
}

#[cfg(not(feature = "test"))]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct OnLoadResult {
  pub content: String,
  #[napi(
    ts_type = r#""dataURI" | "json" | "text" | "css" | "less" | "scss" | "sass" | "js" | "jsx" | "ts" | "tsx" | "null""#
  )]
  pub loader: Option<String>,
}

#[cfg(feature = "test")]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OnLoadResult {
  pub content: String,
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
  pub source: Option<LoadedSource>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct LoadedSource {
  pub content: String,
  pub loader: Option<String>,
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

  async fn build_start(&self, _ctx: &BundleContext) -> () {
    let context = RspackThreadsafeContext::new(());

    let (tx, rx) = oneshot::channel::<()>();

    match REGISTERED_BUILD_START_SENDERS.entry(context.get_call_id()) {
      dashmap::mapref::entry::Entry::Vacant(v) => {
        v.insert(tx);
      }
      dashmap::mapref::entry::Entry::Occupied(_) => {
        self.build_start_tsfn.call(
          Err(Error::new(
            napi::Status::Unknown,
            format!(
              "duplicated call id encountered {}, please file an issue.",
              context.get_call_id(),
            ),
          )),
          ThreadsafeFunctionCallMode::Blocking,
        );
        return;
      }
    }

    let value = serde_json::to_string(&context).map_err(|_| {
      Error::new(
        napi::Status::Unknown,
        "unable to convert context".to_owned(),
      )
    });

    self
      .build_start_tsfn
      .call(value, ThreadsafeFunctionCallMode::Blocking);

    let result = rx.await.expect("failed to receive build_start result");

    result
  }

  async fn build_end(&self, _ctx: &BundleContext) -> () {
    let context = RspackThreadsafeContext::new(());

    let (tx, rx) = oneshot::channel::<()>();

    match REGISTERED_BUILD_END_SENDERS.entry(context.get_call_id()) {
      dashmap::mapref::entry::Entry::Vacant(v) => {
        v.insert(tx);
      }
      dashmap::mapref::entry::Entry::Occupied(_) => {
        self.build_end_tsfn.call(
          Err(Error::new(
            napi::Status::Unknown,
            format!(
              "duplicated call id encountered {}, please file an issue.",
              context.get_call_id(),
            ),
          )),
          ThreadsafeFunctionCallMode::Blocking,
        );
        return;
      }
    }

    let value = serde_json::to_string(&context).map_err(|_| {
      Error::new(
        napi::Status::Unknown,
        "unable to convert context".to_owned(),
      )
    });

    self
      .build_end_tsfn
      .call(value, ThreadsafeFunctionCallMode::Blocking);

    let result = rx.await.expect("failed to receive build_end result");

    result
  }

  #[tracing::instrument(skip_all)]
  async fn resolve(&self, _ctx: &BundleContext, args: &ResolveArgs) -> PluginResolveHookOutput {
    let resolve_context = RspackThreadsafeContext::new(OnResolveContext {
      importer: args.importer.clone(),
      importee: args.id.to_owned(),
    });

    let (tx, rx) = oneshot::channel::<Option<OnResolveResult>>();

    match REGISTERED_RESOLVE_SENDERS.entry(resolve_context.get_call_id()) {
      dashmap::mapref::entry::Entry::Occupied(_) => {
        self.load_tsfn.call(
          Err(Error::new(
            napi::Status::Unknown,
            format!(
              "duplicated call id encountered {}, please file an issue.",
              resolve_context.get_call_id(),
            ),
          )),
          ThreadsafeFunctionCallMode::Blocking,
        );
        return None;
      }
      dashmap::mapref::entry::Entry::Vacant(v) => {
        v.insert(tx);
      }
    }

    let serialized_resolve_context = serde_json::to_string(&resolve_context).map_err(|_| {
      Error::new(
        napi::Status::Unknown,
        "unable to convert context".to_owned(),
      )
    });
    self.resolve_tsfn.call(
      serialized_resolve_context,
      ThreadsafeFunctionCallMode::Blocking,
    );

    let resolve_result = rx.await.expect("failed to receive onresolve result");

    tracing::debug!("[rspack:binding] resolve result {:#?}", resolve_result);

    resolve_result.map(|result| rspack_core::OnResolveResult {
      uri: result.uri,
      external: result.external,
      source: result.source.map(|source| rspack_core::LoadedSource {
        content: source.content,
        loader: source.loader.map(|loader| {
          use rspack_core::Loader;

          match loader.as_str() {
            "dataURI" => Loader::DataURI,
            "json" => Loader::Json,
            "text" => Loader::Text,
            "css" => Loader::Css,
            "less" => Loader::Less,
            "scss" => Loader::Sass,
            "sass" => Loader::Sass,
            "js" => Loader::Js,
            "jsx" => Loader::Jsx,
            "ts" => Loader::Ts,
            "tsx" => Loader::Tsx,
            "null" => Loader::Null,
            _ => panic!("unexpected loader option `{}`", loader),
          }
        }),
      }),
    })
  }

  #[tracing::instrument(skip_all)]
  async fn load(&self, _ctx: &BundleContext, args: &LoadArgs) -> PluginLoadHookOutput {
    let load_context = RspackThreadsafeContext::new(OnLoadContext {
      id: args.id.to_owned(),
    });

    let (tx, rx) = oneshot::channel::<Option<OnLoadResult>>();

    match REGISTERED_LOAD_SENDERS.entry(load_context.get_call_id()) {
      dashmap::mapref::entry::Entry::Vacant(v) => {
        v.insert(tx);
      }
      dashmap::mapref::entry::Entry::Occupied(_) => {
        self.load_tsfn.call(
          Err(Error::new(
            napi::Status::Unknown,
            format!(
              "duplicated call id encountered {}, please file an issue.",
              load_context.get_call_id(),
            ),
          )),
          ThreadsafeFunctionCallMode::Blocking,
        );
        return None;
      }
    }

    let value = serde_json::to_string(&load_context).map_err(|_| {
      Error::new(
        napi::Status::Unknown,
        "unable to convert context".to_owned(),
      )
    });

    self
      .load_tsfn
      .call(value, ThreadsafeFunctionCallMode::Blocking);

    let load_result = rx.await.expect("failed to receive onload result");

    tracing::debug!("[rspack:binding] load result {:#?}", load_result);

    load_result.map(|result| rspack_core::LoadedSource {
      loader: result.loader.map(|loader| {
        use rspack_core::Loader;

        match loader.as_str() {
          "dataURI" => Loader::DataURI,
          "json" => Loader::Json,
          "text" => Loader::Text,
          "css" => Loader::Css,
          "less" => Loader::Less,
          "scss" => Loader::Sass,
          "sass" => Loader::Sass,
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
}
