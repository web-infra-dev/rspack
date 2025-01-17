use std::sync::Arc;

use napi::bindgen_prelude::Either3;
use rspack_collections::DatabaseItem;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_regex::RspackRegex;

use crate::JsChunkWrapper;

pub type Chunks = Either3<RspackRegex, String, ThreadsafeFunction<JsChunkWrapper, bool>>;

pub fn create_chunks_filter(raw: Chunks) -> rspack_plugin_split_chunks::ChunkFilter {
  use pollster::block_on;
  match raw {
    Either3::A(regex) => rspack_plugin_split_chunks::create_regex_chunk_filter_from_str(regex),
    Either3::B(s) => rspack_plugin_split_chunks::create_chunk_filter_from_str(&s),
    Either3::C(f) => Arc::new(move |chunk, compilation| {
      block_on(f.call(JsChunkWrapper::new(chunk.ukey(), compilation)))
    }),
  }
}
