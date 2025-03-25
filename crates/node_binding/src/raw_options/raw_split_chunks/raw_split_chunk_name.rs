use std::sync::Arc;

use napi::bindgen_prelude::Either3;
use napi_derive::napi;
use rspack_collections::DatabaseItem;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_split_chunks::{ChunkNameGetter, ChunkNameGetterFnCtx};

use crate::{JsChunkWrapper, ModuleObject};

pub(super) type RawChunkOptionName =
  Either3<String, bool, ThreadsafeFunction<JsChunkOptionNameCtx, Option<String>>>;

#[inline]
pub(super) fn default_chunk_option_name() -> ChunkNameGetter {
  ChunkNameGetter::Disabled
}

#[napi(object, object_from_js = false)]
pub struct JsChunkOptionNameCtx {
  #[napi(ts_type = "Module")]
  pub module: ModuleObject,
  #[napi(ts_type = "JsChunk[]")]
  pub chunks: Vec<JsChunkWrapper>,
  pub cache_group_key: String,
}

impl<'a> From<ChunkNameGetterFnCtx<'a>> for JsChunkOptionNameCtx {
  fn from(value: ChunkNameGetterFnCtx<'a>) -> Self {
    JsChunkOptionNameCtx {
      module: ModuleObject::new(value.module, value.compilation.id()),
      chunks: value
        .chunks
        .iter()
        .map(|chunk| JsChunkWrapper::new(chunk.ukey(), value.compilation))
        .collect(),
      cache_group_key: value.cache_group_key.to_string(),
    }
  }
}

pub(super) fn normalize_raw_chunk_name(raw: RawChunkOptionName) -> ChunkNameGetter {
  match raw {
    Either3::A(str) => ChunkNameGetter::String(str),
    Either3::B(_) => ChunkNameGetter::Disabled, // FIXME: when set bool is true?
    Either3::C(v) => ChunkNameGetter::Fn(Arc::new(move |ctx: ChunkNameGetterFnCtx| {
      let ctx = ctx.into();
      let v = v.clone();
      Box::pin(async move { v.call(ctx).await })
    })),
  }
}
