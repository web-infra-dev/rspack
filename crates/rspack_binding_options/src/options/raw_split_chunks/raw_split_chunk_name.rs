use std::sync::Arc;

use napi::bindgen_prelude::Either3;
use napi_derive::napi;
use rspack_binding_values::{JsChunk, JsModuleWrapper};
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_split_chunks::{ChunkNameGetter, ChunkNameGetterFnCtx};

pub(super) type RawChunkOptionName =
  Either3<String, bool, ThreadsafeFunction<RawChunkOptionNameCtx, Option<String>>>;

#[inline]
pub(super) fn default_chunk_option_name() -> ChunkNameGetter {
  ChunkNameGetter::Disabled
}

#[napi(object, object_from_js = false)]
pub struct RawChunkOptionNameCtx {
  #[napi(ts_type = "JsModule")]
  pub module: JsModuleWrapper,
  pub chunks: Vec<JsChunk>,
  pub cache_group_key: String,
}

impl<'a> From<ChunkNameGetterFnCtx<'a>> for RawChunkOptionNameCtx {
  fn from(value: ChunkNameGetterFnCtx<'a>) -> Self {
    RawChunkOptionNameCtx {
      module: JsModuleWrapper::new(
        value.module,
        value.compilation.id(),
        Some(value.compilation),
      ),
      chunks: value
        .chunks
        .iter()
        .map(|chunk| JsChunk::from(chunk, value.compilation))
        .collect(),
      cache_group_key: value.cache_group_key.to_string(),
    }
  }
}

pub(super) fn normalize_raw_chunk_name(raw: RawChunkOptionName) -> ChunkNameGetter {
  use pollster::block_on;
  match raw {
    Either3::A(str) => ChunkNameGetter::String(str),
    Either3::B(_) => ChunkNameGetter::Disabled, // FIXME: when set bool is true?
    Either3::C(v) => ChunkNameGetter::Fn(Arc::new(move |ctx| block_on(v.call(ctx.into())))),
  }
}
