use std::sync::Arc;

use rspack_core::{Compilation, Module};
use rspack_error::Result;

pub struct CacheGroupTestFnCtx<'a> {
  pub compilation: &'a Compilation,
  pub module: &'a dyn Module,
}

type CacheGroupTestFn = Arc<dyn Fn(CacheGroupTestFnCtx<'_>) -> Result<Option<bool>> + Send + Sync>;

#[derive(Clone)]
pub enum CacheGroupTest {
  String(String),
  Fn(CacheGroupTestFn),
  RegExp(rspack_regex::RspackRegex),
  Enabled,
}
