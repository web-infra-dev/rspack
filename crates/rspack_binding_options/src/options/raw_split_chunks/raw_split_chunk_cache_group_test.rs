use std::sync::Arc;

use napi::bindgen_prelude::{Either3, FromNapiValue};
use napi_derive::napi;
use rspack_binding_values::JsModuleWrapper;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_napi::JsRegExp;
use rspack_plugin_split_chunks::{CacheGroupTest, CacheGroupTestFnCtx};

pub(super) type RawCacheGroupTest =
  Either3<String, JsRegExp, ThreadsafeFunction<JsCacheGroupTestCtx, Option<bool>>>;

#[napi(object, object_from_js = false)]
pub struct JsCacheGroupTestCtx {
  #[napi(ts_type = "JsModule")]
  pub module: JsModuleWrapper,
}

impl FromNapiValue for JsCacheGroupTestCtx {
  unsafe fn from_napi_value(
    _env: napi::sys::napi_env,
    _napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    unreachable!()
  }
}

impl<'a> From<CacheGroupTestFnCtx<'a>> for JsCacheGroupTestCtx {
  fn from(value: CacheGroupTestFnCtx<'a>) -> Self {
    JsCacheGroupTestCtx {
      module: JsModuleWrapper::new(value.module, Some(value.compilation)),
    }
  }
}

pub(super) fn normalize_raw_cache_group_test(raw: RawCacheGroupTest) -> CacheGroupTest {
  use pollster::block_on;
  match raw {
    Either3::A(str) => CacheGroupTest::String(str),
    Either3::B(regexp) => CacheGroupTest::RegExp(regexp.into()),
    Either3::C(v) => CacheGroupTest::Fn(Arc::new(move |ctx| handle.block_on(v.call(ctx.into())))),
  }
}

#[inline]
pub(super) fn default_cache_group_test() -> CacheGroupTest {
  CacheGroupTest::Enabled
}
