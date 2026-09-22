use std::sync::Arc;

use futures::future::BoxFuture;
use rspack_core::{Compilation, Module};
use rspack_error::Result;
#[cfg(allocative)]
use rspack_util::allocative;

pub struct CacheGroupTestFnCtx<'a> {
  pub compilation: &'a Compilation,
  pub module: &'a dyn Module,
}

type CacheGroupTestFn =
  Arc<dyn Fn(CacheGroupTestFnCtx<'_>) -> BoxFuture<'static, Result<Option<bool>>> + Send + Sync>;

#[derive(Clone)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub enum CacheGroupTest {
  String(String),
  Fn(#[cfg_attr(allocative, allocative(visit = allocative::visit_opaque_arc))] CacheGroupTestFn),
  RegExp(rspack_regex::RspackRegex),
  Enabled,
}
