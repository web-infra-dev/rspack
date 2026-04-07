use std::sync::Arc;

use napi::bindgen_prelude::Either3;
use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_split_chunks::{CacheGroupTest, CacheGroupTestFnCtx};

use crate::{js_regex::JsRegExp, module::ModuleObject};

pub(super) type RawCacheGroupTest =
  Either3<String, JsRegExp, ThreadsafeFunction<JsCacheGroupTestCtx, Option<bool>>>;

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

pub(super) fn normalize_raw_cache_group_test(
  raw: RawCacheGroupTest,
) -> rspack_error::Result<CacheGroupTest> {
  Ok(match raw {
    Either3::A(str) => CacheGroupTest::String(str),
    Either3::B(regexp) => CacheGroupTest::RegExp(regexp.try_into()?),
    Either3::C(v) => CacheGroupTest::Fn(Arc::new(move |ctx| {
      let ctx = ctx.into();
      let v = v.clone();
      Box::pin(async move { v.call_with_sync(ctx).await })
    })),
  })
}

#[inline]
pub(super) fn default_cache_group_test() -> CacheGroupTest {
  CacheGroupTest::Enabled
}
