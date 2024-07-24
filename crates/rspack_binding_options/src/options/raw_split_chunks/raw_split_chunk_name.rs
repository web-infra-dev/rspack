use std::sync::Arc;

use napi::bindgen_prelude::Either3;
use napi_derive::napi;
use rspack_binding_values::{JsChunk, JsModule, ToJsModule};
use rspack_core::Chunk;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_split_chunks::{ChunkNameGetter, ChunkNameGetterFnCtx};
use tokio::runtime::Handle;

pub(super) type RawChunkOptionName =
  Either3<String, bool, ThreadsafeFunction<RawChunkOptionNameCtx, Option<String>>>;

#[inline]
pub(super) fn default_chunk_option_name() -> ChunkNameGetter {
  ChunkNameGetter::Disabled
}

#[napi(object)]
pub struct RawChunkOptionNameCtx {
  pub module: JsModule,
  pub chunks: Vec<JsChunk>,
  pub cache_group_key: String,
}

impl<'a> From<ChunkNameGetterFnCtx<'a>> for RawChunkOptionNameCtx {
  fn from(value: ChunkNameGetterFnCtx<'a>) -> Self {
    RawChunkOptionNameCtx {
      module: value
        .module
        .to_js_module()
        .expect("should convert js success"),
      chunks: value
        .chunks
        .iter()
        .map(|chunk: &&Chunk| JsChunk::from(chunk))
        .collect(),
      cache_group_key: value.cache_group_key.to_string(),
    }
  }
}

pub(super) fn normalize_raw_chunk_name(raw: RawChunkOptionName) -> ChunkNameGetter {
  let handle = Handle::current();
  match raw {
    Either3::A(str) => ChunkNameGetter::String(str),
    Either3::B(_) => ChunkNameGetter::Disabled, // FIXME: when set bool is true?
    Either3::C(v) => ChunkNameGetter::Fn(Arc::new(move |ctx| handle.block_on(v.call(ctx.into())))),
  }
}
