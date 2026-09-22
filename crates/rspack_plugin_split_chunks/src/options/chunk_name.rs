use std::sync::Arc;

use futures::future::BoxFuture;
use rspack_core::{ChunkUkey, Compilation, Module};
use rspack_error::Result;
#[cfg(allocative)]
use rspack_util::allocative;

pub struct ChunkNameGetterFnCtx<'a> {
  pub module: &'a dyn Module,
  pub compilation: &'a Compilation,
  pub chunks: &'a Vec<ChunkUkey>,
  pub cache_group_key: &'a str,
}

type ChunkNameGetterFn = Arc<
  dyn for<'a> Fn(ChunkNameGetterFnCtx<'a>) -> BoxFuture<'static, Result<Option<String>>>
    + Sync
    + Send,
>;

#[derive(Clone)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub enum ChunkNameGetter {
  String(String),
  Fn(#[cfg_attr(allocative, allocative(visit = allocative::visit_opaque_arc))] ChunkNameGetterFn),
  Disabled,
}
