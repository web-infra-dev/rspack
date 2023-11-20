use std::sync::Arc;

use futures_util::future::BoxFuture;
use rspack_core::Module;

pub struct ChunkNameGetterFnCtx<'a> {
  pub module: &'a dyn Module,
}

type ChunkNameGetterFn =
  Arc<dyn for<'a> Fn(ChunkNameGetterFnCtx<'a>) -> BoxFuture<'a, Option<String>> + Send + Sync>;

#[derive(Clone)]
pub enum ChunkNameGetter {
  String(String),
  Fn(ChunkNameGetterFn),
  Disabled,
}
