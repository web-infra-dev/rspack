use std::sync::Arc;

use napi::bindgen_prelude::Either3;
use napi_derive::napi;
use rspack_binding_values::ModuleDTOWrapper;
use rspack_napi::regexp::{JsRegExp, JsRegExpExt};
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_split_chunks::{CacheGroupTest, CacheGroupTestFnCtx};
use tokio::runtime::Handle;

pub(super) type RawCacheGroupTest =
  Either3<String, JsRegExp, ThreadsafeFunction<JsCacheGroupTestCtx, Option<bool>>>;

#[napi(object)]
pub struct JsCacheGroupTestCtx {
  pub module: ModuleDTOWrapper,
}

impl<'a> From<CacheGroupTestFnCtx<'a>> for JsCacheGroupTestCtx {
  fn from(value: CacheGroupTestFnCtx<'a>) -> Self {
    JsCacheGroupTestCtx {
      module: ModuleDTOWrapper::new(value.module.identifier(), &value.compilation),
    }
  }
}

pub(super) fn normalize_raw_cache_group_test(raw: RawCacheGroupTest) -> CacheGroupTest {
  let handle = Handle::current();
  match raw {
    Either3::A(str) => CacheGroupTest::String(str),
    Either3::B(regexp) => CacheGroupTest::RegExp(regexp.to_rspack_regex()),
    Either3::C(v) => CacheGroupTest::Fn(Arc::new(move |ctx| handle.block_on(v.call(ctx.into())))),
  }
}

#[inline]
pub(super) fn default_cache_group_test() -> CacheGroupTest {
  CacheGroupTest::Enabled
}
