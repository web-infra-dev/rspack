use std::collections::HashMap;
use std::fmt::Debug;

use rspack_core::rspack_sources::{RawSource, SourceExt};
use rspack_core::{
  Compilation, Plugin, PluginBuildEndHookOutput, PluginContext, PluginProcessAssetsHookOutput,
  ProcessAssetsArgs,
};

use async_trait::async_trait;
use napi::{CallContext, JsUndefined};
use napi_derive::napi;
use serde::{Deserialize, Serialize};

pub mod utils;
pub use utils::create_node_adapter_from_plugin_callbacks;

use crate::{AssetContent, UpdateAssetOptions};

pub type BoxedClosure = Box<dyn Fn(CallContext<'_>) -> napi::Result<JsUndefined>>;

pub struct RspackPluginNodeAdapter {
  pub done_tsfn: crate::threadsafe_function::ThreadsafeFunction<(), ()>,
  pub process_assets_tsfn:
    crate::threadsafe_function::ThreadsafeFunction<(String, BoxedClosure), ()>,
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
  async fn process_assets(
    &mut self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsHookOutput {
    let assets: HashMap<String, Vec<u8>> = args
      .compilation
      .assets
      .iter()
      .map(|asset| (asset.0.clone(), asset.1.get_source().buffer().to_vec()))
      .collect();

    let rx = {
      let compilation = args.compilation as *mut Compilation;

      let emit_asset = move |call_context: CallContext<'_>| {
        let options = call_context.get::<UpdateAssetOptions>(0)?;

        // Safety: since the `compilation` will available throughout the build procedure, this operation is always considered safe under the `processAsset` hook.
        // For developers who use `@rspack/binding` under the hood, exposing a reference to the `emit_asset` fn to user side is strongly not recommended since `compilation` may be dropped.
        let compilation = unsafe { &mut *compilation.cast::<Compilation>() };
        let asset = compilation.assets.get_mut(&options.filename).unwrap();
        asset.set_source(match options.asset {
          AssetContent {
            buffer: Some(buffer),
            source: None,
          } => RawSource::Buffer(buffer.into()).boxed(),
          AssetContent {
            buffer: None,
            source: Some(source),
          } => RawSource::Source(source).boxed(),
          _ => panic!("AssetContent can only be string or buffer"),
        });

        call_context.env.get_undefined()
      };

      let value = serde_json::to_string(&assets)
        .map_err(|_| rspack_error::Error::InternalError("Failed to stringify assets".to_owned()))?;

      self
        .process_assets_tsfn
        .call(
          (value, Box::new(emit_asset) as BoxedClosure),
          crate::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
        )
        .map_err(rspack_error::Error::from)?
    };

    rx.await
      .map_err(|err| rspack_error::Error::InternalError(format!("{:?}", err)))
  }

  #[tracing::instrument(skip_all)]
  async fn done(&mut self) -> PluginBuildEndHookOutput {
    self
      .done_tsfn
      .call(
        (),
        crate::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
      )
      .map_err(rspack_error::Error::from)?
      .await
      .map_err(|err| rspack_error::Error::InternalError(format!("{:?}", err)))?;

    Ok(())
  }
}
