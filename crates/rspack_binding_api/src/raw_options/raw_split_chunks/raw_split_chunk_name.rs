use std::sync::Arc;

use napi::bindgen_prelude::Either4;
use napi_derive::napi;
use rspack_plugin_split_chunks::{ChunkNameGetter, ChunkNameGetterFnCtx};

use crate::{
  chunk::ChunkWrapper,
  compiler_scoped_tsfn::CompilerScopedTsFnHandle as ThreadsafeFunction,
  module::ModuleObject,
  worker::{JsWorkerFunction, SplitChunkNameTaskPayload, WorkerTaskPayload, dispatch_worker_task},
};

pub(super) type RawChunkOptionName =
  Either4<String, bool, ThreadsafeFunction<JsChunkOptionNameCtx, Option<String>>, JsWorkerFunction>;

#[inline]
pub(super) fn default_chunk_option_name() -> ChunkNameGetter {
  ChunkNameGetter::Disabled
}

#[napi(object, object_from_js = false)]
pub struct JsChunkOptionNameCtx {
  #[napi(ts_type = "Module")]
  pub module: ModuleObject,
  #[napi(ts_type = "Chunk[]")]
  pub chunks: Vec<ChunkWrapper>,
  pub cache_group_key: String,
}

impl<'a> From<ChunkNameGetterFnCtx<'a>> for JsChunkOptionNameCtx {
  fn from(value: ChunkNameGetterFnCtx<'a>) -> Self {
    JsChunkOptionNameCtx {
      module: ModuleObject::with_ref(value.module, value.compilation.compiler_id()),
      chunks: value
        .chunks
        .iter()
        .map(|chunk| ChunkWrapper::new(*chunk, value.compilation))
        .collect(),
      cache_group_key: value.cache_group_key.to_string(),
    }
  }
}

pub(super) fn normalize_raw_chunk_name(raw: RawChunkOptionName) -> ChunkNameGetter {
  match raw {
    Either4::A(str) => ChunkNameGetter::String(str),
    Either4::B(_) => ChunkNameGetter::Disabled, // FIXME: when set bool is true?
    Either4::C(v) => ChunkNameGetter::Fn(Arc::new(move |ctx: ChunkNameGetterFnCtx| {
      let ctx = ctx.into();
      let v = v.clone();
      Box::pin(async move { v.call_with_sync(ctx).await })
    })),
    Either4::D(function) => ChunkNameGetter::Fn(Arc::new(move |ctx: ChunkNameGetterFnCtx| {
      // Materialize data while the compilation is borrowed, before entering the worker queue.
      let data = serde_json::json!({
        "module": {
          "identifier": ctx.module.identifier().to_string(),
          "nameForCondition": ctx.module.name_for_condition(),
          "type": ctx.module.module_type().to_string(),
          "layer": ctx.module.get_layer(),
        },
        "chunks": ctx.chunks.iter().map(|key| {
          serde_json::json!({ "name": ctx.compilation.build_chunk_graph_artifact.chunk_by_ukey.expect_get(key).name() })
        }).collect::<Vec<_>>(),
        "cacheGroupKey": ctx.cache_group_key,
      }).to_string();
      let function = function.clone();
      Box::pin(async move {
        if function.version != 1 || function.hook != "optimization.splitChunks.name" {
          return Err(rspack_error::error!(
            "Unsupported workerFunction splitChunks.name codec"
          ));
        }
        let payload = dispatch_worker_task(Box::new(WorkerTaskPayload::SplitChunkName(
          SplitChunkNameTaskPayload {
            function,
            data: Some(data),
            result: None,
          },
        )))
        .await
        .map_err(|failure| {
          let (error, _) = failure.into_parts();
          rspack_error::error!(error.to_string())
        })?;
        let WorkerTaskPayload::SplitChunkName(payload) = *payload else {
          unreachable!("splitChunks.name task must return its naming result")
        };
        Ok(payload.result)
      })
    })),
  }
}
