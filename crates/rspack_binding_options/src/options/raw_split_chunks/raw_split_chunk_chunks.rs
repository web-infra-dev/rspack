use std::sync::Arc;

use napi::Env;
use napi::{bindgen_prelude::Either3, JsFunction, JsString};
use rspack_binding_values::JsChunk;
use rspack_napi_shared::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use rspack_napi_shared::{get_napi_env, NapiResultExt};
use rspack_napi_shared::{JsRegExp, JsRegExpExt, JsStringExt};

pub type Chunks = Either3<JsRegExp, JsString, JsFunction>;

pub fn create_chunks_filter(raw: Chunks) -> rspack_plugin_split_chunks_new::ChunkFilter {
  match raw {
    Either3::A(reg) => {
      rspack_plugin_split_chunks_new::create_regex_chunk_filter_from_str(reg.to_rspack_regex())
    }
    Either3::B(js_str) => {
      let js_str = js_str.into_string();
      rspack_plugin_split_chunks_new::create_chunk_filter_from_str(&js_str)
    }
    Either3::C(f) => {
      let fn_payload: napi::Result<ThreadsafeFunction<JsChunk, bool>> = try {
        let env = get_napi_env();
        rspack_binding_macros::js_fn_into_threadsafe_fn!(f, &Env::from(env))
      };
      let fn_payload = Arc::new(fn_payload.expect("convert to threadsafe function failed"));
      Arc::new(move |chunk, _| {
        let fn_payload = fn_payload.clone();
        fn_payload
          .call(
            JsChunk::from(chunk),
            ThreadsafeFunctionCallMode::NonBlocking,
          )
          .into_rspack_result()
          .expect("into rspack result failed")
          .blocking_recv()
          .unwrap_or_else(|err| panic!("Failed to call external function: {err}"))
          .expect("failed")
      })
    }
  }
}
