use std::sync::Arc;

use napi::bindgen_prelude::Either3;
use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_split_chunks::{CacheGroupTest, CacheGroupTestFnCtx};
use rspack_regex::RspackRegex;

use crate::ModuleObject;

pub(super) type RawCacheGroupTest =
  Either3<String, RspackRegex, ThreadsafeFunction<JsCacheGroupTestCtx, Option<bool>>>;

#[napi(object, object_from_js = false)]
pub struct JsCacheGroupTestCtx {
  #[napi(ts_type = "Module")]
  pub module: ModuleObject,
}

impl<'a> From<CacheGroupTestFnCtx<'a>> for JsCacheGroupTestCtx {
  fn from(value: CacheGroupTestFnCtx<'a>) -> Self {
    JsCacheGroupTestCtx {
      module: ModuleObject::with_ref(value.module, value.compilation.compiler_id()),
    }
  }
}

pub(super) fn normalize_raw_cache_group_test(raw: RawCacheGroupTest) -> CacheGroupTest {
  use pollster::block_on;
  match raw {
    Either3::A(str) => CacheGroupTest::String(str),
    Either3::B(regexp) => CacheGroupTest::RegExp(regexp),
    Either3::C(v) => CacheGroupTest::Fn(Arc::new(move |ctx| block_on(v.call(ctx.into())))),
  }
}

#[inline]
pub(super) fn default_cache_group_test() -> CacheGroupTest {
  CacheGroupTest::Enabled
}
