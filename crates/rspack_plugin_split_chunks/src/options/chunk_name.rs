use std::sync::Arc;

use rspack_core::Module;
use rspack_error::Result;

pub struct ChunkNameGetterFnCtx<'a> {
  pub module: &'a dyn Module,
}

type ChunkNameGetterFn =
  Arc<dyn for<'a> Fn(ChunkNameGetterFnCtx<'a>) -> Result<Option<String>> + Send + Sync>;

#[derive(Clone)]
pub enum ChunkNameGetter {
  String(String),
  Fn(ChunkNameGetterFn),
  Disabled,
}
