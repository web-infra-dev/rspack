use std::sync::Arc;

use napi::bindgen_prelude::Either3;
use napi::{Env, JsFunction};
use napi_derive::napi;
use rspack_binding_values::{JsModule, ToJsModule};
use rspack_napi_shared::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use rspack_napi_shared::{get_napi_env, NapiResultExt};
use rspack_plugin_split_chunks_new::{ChunkNameGetter, ChunkNameGetterFnCtx};

pub(super) type RawChunkOptionName = Either3<String, bool, JsFunction>;

#[inline]
pub(super) fn default_chunk_option_name() -> ChunkNameGetter {
  ChunkNameGetter::Disabled
}

#[napi(object)]
struct RawChunkOptionNameCtx {
  pub module: JsModule,
}

impl<'a> From<ChunkNameGetterFnCtx<'a>> for RawChunkOptionNameCtx {
  fn from(value: ChunkNameGetterFnCtx<'a>) -> Self {
    RawChunkOptionNameCtx {
      module: value
        .module
        .to_js_module()
        .expect("should convert js success"),
    }
  }
}

pub(super) fn normalize_raw_chunk_name(raw: RawChunkOptionName) -> ChunkNameGetter {
  match raw {
    Either3::A(str) => ChunkNameGetter::String(str),
    Either3::B(_) => ChunkNameGetter::Disabled, // FIXME: when set bool is true?
    Either3::C(v) => {
      let fn_payload: napi::Result<ThreadsafeFunction<RawChunkOptionNameCtx, Option<String>>> = try {
        let env = get_napi_env();
        rspack_binding_macros::js_fn_into_threadsafe_fn!(v, &Env::from(env))
      };
      let fn_payload = Arc::new(fn_payload.expect("convert to threadsafe function failed"));
      ChunkNameGetter::Fn(Arc::new(move |ctx| {
        let fn_payload = fn_payload.clone();
        fn_payload
          .call(ctx.into(), ThreadsafeFunctionCallMode::NonBlocking)
          .into_rspack_result()
          .expect("into rspack result failed")
          .blocking_recv()
          .unwrap_or_else(|err| panic!("Failed to call external function: {err}"))
          .expect("failed")
      }))
    }
  }
}
