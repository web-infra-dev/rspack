use std::collections::HashMap;
use std::fmt::Debug;
use std::pin::Pin;

use napi_derive::napi;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use rspack_core::{
  Compilation, CompilationArgs, DoneArgs, Plugin, PluginBuildEndHookOutput,
  PluginCompilationHookOutput, PluginContext, PluginProcessAssetsHookOutput,
  PluginThisCompilationHookOutput, ProcessAssetsArgs, ThisCompilationArgs,
};
use rspack_error::Error;

use crate::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};

use crate::{JsCompatSource, JsCompilation, StatsCompilation, ToJsCompatSource};

mod utils;
pub use utils::*;

pub struct RspackPluginNodeAdapter {
  pub done_tsfn: ThreadsafeFunction<StatsCompilation, ()>,
  pub compilation_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub this_compilation_tsfn: ThreadsafeFunction<JsCompilation, ()>,
  pub process_assets_tsfn: ThreadsafeFunction<HashMap<String, JsCompatSource>, ()>,
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
  fn compilation(&mut self, args: CompilationArgs) -> PluginCompilationHookOutput {
    let compilation = JsCompilation::from_compilation(unsafe {
      Pin::new_unchecked(std::mem::transmute::<
        &'_ mut Compilation,
        &'static mut Compilation,
      >(args.compilation))
    });

    self
      .compilation_tsfn
      .call(compilation, ThreadsafeFunctionCallMode::Blocking)?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  fn this_compilation(&mut self, args: ThisCompilationArgs) -> PluginThisCompilationHookOutput {
    let compilation = JsCompilation::from_compilation(unsafe {
      Pin::new_unchecked(std::mem::transmute::<
        &'_ mut Compilation,
        &'static mut Compilation,
      >(args.this_compilation))
    });

    self
      .this_compilation_tsfn
      .call(compilation, ThreadsafeFunctionCallMode::Blocking)?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn process_assets(
    &mut self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsHookOutput {
    let mut assets = HashMap::<String, JsCompatSource>::new();

    for (filename, asset) in &args.compilation.assets {
      let source = asset.source.as_ref().to_js_compat_source()?;
      assets.insert(filename.clone(), source);
    }

    let rx = self
      .process_assets_tsfn
      .call(assets, ThreadsafeFunctionCallMode::Blocking)
      .map_err(Error::from)?;

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
