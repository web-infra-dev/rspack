use std::sync::Arc;

use napi::bindgen_prelude::Either3;
use napi::{Env, JsFunction};
use napi_derive::napi;
use rspack_binding_values::{JsModule, ToJsModule};
use rspack_napi_shared::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use rspack_napi_shared::{JsRegExp, JsRegExpExt, NapiResultExt, NAPI_ENV};
use rspack_plugin_split_chunks_new::{CacheGroupTest, CacheGroupTestFnCtx};

pub(super) type RawCacheGroupTest = Either3<String, JsRegExp, JsFunction>;

#[napi(object)]
struct RawCacheGroupTestCtx {
  pub module: JsModule,
}

impl<'a> From<CacheGroupTestFnCtx<'a>> for RawCacheGroupTestCtx {
  fn from(value: CacheGroupTestFnCtx<'a>) -> Self {
    RawCacheGroupTestCtx {
      module: value
        .module
        .to_js_module()
        .expect("should convert js module success"),
    }
  }
}

pub(super) fn normalize_raw_cache_group_test(raw: RawCacheGroupTest) -> CacheGroupTest {
  match raw {
    Either3::A(str) => CacheGroupTest::String(str),
    Either3::B(regexp) => CacheGroupTest::RegExp(regexp.to_rspack_regex()),
    Either3::C(v) => {
      let fn_payload: ThreadsafeFunction<RawCacheGroupTestCtx, Option<bool>> = NAPI_ENV
        .with(|env| -> anyhow::Result<_> {
          let env = env.borrow().expect("Failed to get env with external");
          let fn_payload = rspack_binding_macros::js_fn_into_threadsafe_fn!(v, &Env::from(env));
          Ok(fn_payload)
        })
        .expect("should generate fn_payload success");
      let fn_payload = Arc::new(fn_payload);
      CacheGroupTest::Fn(Arc::new(move |ctx| {
        let fn_payload = fn_payload.clone();
        Box::pin(async move {
          fn_payload
            .call(ctx.into(), ThreadsafeFunctionCallMode::NonBlocking)
            .into_rspack_result()
            .expect("into rspack result failed")
            .await
            .unwrap_or_else(|err| panic!("Failed to call external function: {err}"))
            .expect("failed")
        })
      }))
    }
  }
}

#[inline]
pub(super) fn default_cache_group_test() -> CacheGroupTest {
  CacheGroupTest::Enabled
}
