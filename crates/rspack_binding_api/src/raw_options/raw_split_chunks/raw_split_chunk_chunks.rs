use std::sync::Arc;

use napi::{JsString, bindgen_prelude::Either3};
use rspack_napi::{string::JsStringExt, threadsafe_function::ThreadsafeFunction};
use rspack_plugin_split_chunks::{
  ChunkFilter, create_chunk_filter_from_str, create_regex_chunk_filter_from_str,
};
use rspack_regex::RspackRegex;

use crate::chunk::ChunkWrapper;

pub type Chunks<'a> = Either3<RspackRegex, JsString<'a>, ThreadsafeFunction<ChunkWrapper, bool>>;

pub fn create_chunks_filter(raw: Chunks) -> ChunkFilter {
  match raw {
    Either3::A(regex) => create_regex_chunk_filter_from_str(regex),
    Either3::B(js_str) => {
      let js_str = js_str.into_string();
      create_chunk_filter_from_str(&js_str)
    }
    Either3::C(f) => ChunkFilter::Func(Arc::new(move |chunk_ukey, compilation| {
      let f = f.clone();
      let chunk_wrapper = ChunkWrapper::new(*chunk_ukey, compilation);
      Box::pin(async move { f.call_with_sync(chunk_wrapper).await })
    })),
  }
}
