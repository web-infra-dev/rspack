use std::sync::Arc;

use napi::{bindgen_prelude::Either3, JsString};
use rspack_binding_values::JsChunk;
use rspack_napi::regexp::{JsRegExp, JsRegExpExt};
use rspack_napi::string::JsStringExt;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use tokio::runtime::Handle;

pub type Chunks = Either3<JsRegExp, JsString, ThreadsafeFunction<JsChunk, bool>>;

pub fn create_chunks_filter(raw: Chunks) -> rspack_plugin_split_chunks::ChunkFilter {
  let handle = Handle::current();
  match raw {
    Either3::A(reg) => {
      rspack_plugin_split_chunks::create_regex_chunk_filter_from_str(reg.to_rspack_regex())
    }
    Either3::B(js_str) => {
      let js_str = js_str.into_string();
      rspack_plugin_split_chunks::create_chunk_filter_from_str(&js_str)
    }
    Either3::C(f) => Arc::new(move |chunk, _| handle.block_on(f.call(JsChunk::from(chunk)))),
  }
}
