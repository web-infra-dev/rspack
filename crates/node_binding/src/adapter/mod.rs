use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};

use rspack_core::{Plugin, PluginBuildEndHookOutput};

use anyhow::Context;
use async_trait::async_trait;
use napi::threadsafe_function::ThreadsafeFunctionCallMode;
use napi::Error;
use napi_derive::napi;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

mod common;
pub mod utils;
pub use utils::create_node_adapter_from_plugin_callbacks;

use common::ThreadsafeRspackCallback;
use common::REGISTERED_BUILD_END_SENDERS;

pub static CALL_ID: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(1));

pub struct RspackPluginNodeAdapter {
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
  async fn build_end(&self) -> PluginBuildEndHookOutput {
    let context = RspackThreadsafeContext::new(());

    let (tx, rx) = oneshot::channel::<()>();

    match REGISTERED_BUILD_END_SENDERS.entry(context.get_call_id()) {
      dashmap::mapref::entry::Entry::Vacant(v) => {
        v.insert(tx);
      }
      dashmap::mapref::entry::Entry::Occupied(_) => {
        let err = Error::new(
          napi::Status::Unknown,
          format!(
            "duplicated call id encountered {}, please file an issue.",
            context.get_call_id(),
          ),
        );
        self
          .build_end_tsfn
          .call(Err(err.clone()), ThreadsafeFunctionCallMode::Blocking);

        let any_error = anyhow::Error::from(err);
        return Err(any_error.into());
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

    let t = rx
      .await
      .context("failed to receive build_end result")
      .map_err(|err| err.into());
    return t;
  }
}
