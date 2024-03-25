use std::sync::Arc;

use napi::bindgen_prelude::Either3;
use napi_derive::napi;
use rspack_binding_values::{JsModule, ToJsModule};
use rspack_napi::regexp::{JsRegExp, JsRegExpExt};
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_split_chunks::{CacheGroupTest, CacheGroupTestFnCtx};
use tokio::runtime::Handle;

pub(super) type RawCacheGroupTest =
  Either3<String, JsRegExp, ThreadsafeFunction<RawCacheGroupTestCtx, Option<bool>>>;

#[napi(object)]
pub struct RawCacheGroupTestCtx {
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
  let handle = Handle::current();
  match raw {
    Either3::A(str) => CacheGroupTest::String(str),
    Either3::B(regexp) => CacheGroupTest::RegExp(regexp.to_rspack_regex()),
    Either3::C(v) => CacheGroupTest::Fn(Arc::new(move |ctx| {
      handle
        .block_on(v.call(ctx.into()))
        .expect("failed to load cache group test")
    })),
  }
}

#[inline]
pub(super) fn default_cache_group_test() -> CacheGroupTest {
  CacheGroupTest::Enabled
}
