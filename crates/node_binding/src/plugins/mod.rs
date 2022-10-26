use std::collections::HashMap;
use std::fmt::Debug;

use napi::{CallContext, JsUndefined};
use napi_derive::napi;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use rspack_core::rspack_sources::{RawSource, SourceExt};
use rspack_core::{
  Compilation, DoneArgs, Plugin, PluginBuildEndHookOutput, PluginContext,
  PluginProcessAssetsHookOutput, ProcessAssetsArgs,
};
use rspack_error::Error;

use crate::js_values::{RspackCompilation, StatsCompilation};
use crate::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use crate::{AssetContent, UpdateAssetOptions};

mod utils;
pub use utils::*;

pub type BoxedClosure = Box<dyn Fn(CallContext<'_>) -> napi::Result<JsUndefined>>;

pub struct RspackPluginNodeAdapter {
  pub done_tsfn: ThreadsafeFunction<StatsCompilation, ()>,
  pub process_assets_tsfn: ThreadsafeFunction<(String, BoxedClosure), ()>,
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
        .map_err(|_| Error::InternalError("Failed to stringify assets".to_owned()))?;

      self
        .process_assets_tsfn
        .call(
          (value, Box::new(emit_asset) as BoxedClosure),
          ThreadsafeFunctionCallMode::Blocking,
        )
        .map_err(Error::from)?
    };

    rx.await
      .map_err(|err| Error::InternalError(format!("{:?}", err)))
  }

  #[tracing::instrument(skip_all)]
  async fn done<'s, 'c>(
    &mut self,
    _ctx: PluginContext,
    args: DoneArgs<'s, 'c>,
  ) -> PluginBuildEndHookOutput {
    self
      .done_tsfn
      .call(
        args.stats.to_description().into(),
        ThreadsafeFunctionCallMode::Blocking,
      )
      .map_err(Error::from)?
      .await
      .map_err(|err| Error::InternalError(format!("{:?}", err)))?;

    Ok(())
  }
}
