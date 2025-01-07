use std::sync::Arc;

use derive_more::Debug;
use futures::future::BoxFuture;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_error::Result;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_rsdoctor::{
  RsdoctorChunkGraph, RsdoctorModuleGraph, RsdoctorPluginOptions, SendChunkGraph, SendModuleGraph,
};

use super::{JsRsdoctorChunkGraph, JsRsdoctorModuleGraph};

pub type RawSendModuleGraphFn = ThreadsafeFunction<JsRsdoctorModuleGraph, ()>;
pub type RawSendChunkGraphFn = ThreadsafeFunction<JsRsdoctorChunkGraph, ()>;

#[napi(object, object_to_js = false)]
pub struct RawRsdoctorPluginOptions {
  #[napi(ts_type = "(moduleGraph: JsRsdoctorModuleGraph) => Promise<void>")]
  pub on_module_graph: Option<RawSendModuleGraphFn>,
  #[napi(ts_type = "(chunkGraph: JsRsdoctorChunkGraph) => Promise<void>")]
  pub on_chunk_graph: Option<RawSendChunkGraphFn>,
}

impl From<RawRsdoctorPluginOptions> for RsdoctorPluginOptions {
  fn from(value: RawRsdoctorPluginOptions) -> Self {
    let on_module_graph = value.on_module_graph.map(|func| -> SendModuleGraph {
      Arc::new(
        move |module_graph: RsdoctorModuleGraph| -> BoxFuture<'static, Result<()>> {
          let f = func.clone();
          Box::pin(async move { f.call(module_graph.into()).await })
        },
      ) as _
    });

    let on_chunk_graph = value.on_chunk_graph.map(|func| -> SendChunkGraph {
      Arc::new(
        move |chunk_graph: RsdoctorChunkGraph| -> BoxFuture<'static, Result<()>> {
          let f = func.clone();
          Box::pin(async move { f.call(chunk_graph.into()).await })
        },
      ) as _
    });

    Self {
      on_module_graph,
      on_chunk_graph,
    }
  }
}
