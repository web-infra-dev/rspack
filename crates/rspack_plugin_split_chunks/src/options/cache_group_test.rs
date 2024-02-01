use std::sync::Arc;

use rspack_core::Module;

pub struct CacheGroupTestFnCtx<'a> {
  pub module: &'a dyn Module,
}

type CacheGroupTestFn = Arc<dyn for<'a> Fn(CacheGroupTestFnCtx<'a>) -> Option<bool> + Send + Sync>;

#[derive(Clone)]
pub enum CacheGroupTest {
  String(String),
  Fn(CacheGroupTestFn),
  RegExp(rspack_regex::RspackRegex),
  Enabled,
}
