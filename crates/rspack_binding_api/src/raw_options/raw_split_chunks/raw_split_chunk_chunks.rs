use std::sync::Arc;

use napi::{JsString, bindgen_prelude::Either3};
use rspack_collections::DatabaseItem;
use rspack_napi::{string::JsStringExt, threadsafe_function::ThreadsafeFunction};
use rspack_regex::RspackRegex;

use crate::ChunkWrapper;

pub type Chunks<'a> = Either3<RspackRegex, JsString<'a>, ThreadsafeFunction<ChunkWrapper, bool>>;

pub fn create_chunks_filter(raw: Chunks) -> rspack_plugin_split_chunks::ChunkFilter {
  match raw {
    Either3::A(regex) => rspack_plugin_split_chunks::create_regex_chunk_filter_from_str(regex),
    Either3::B(js_str) => {
      let js_str = js_str.into_string();
      rspack_plugin_split_chunks::create_chunk_filter_from_str(&js_str)
    }
    Either3::C(f) => Arc::new(move |chunk, compilation| {
      let f = f.clone();
      let chunk_wrapper = ChunkWrapper::new(chunk.ukey(), compilation);
      Box::pin(async move { f.call_with_sync(chunk_wrapper).await })
    }),
  }
}
