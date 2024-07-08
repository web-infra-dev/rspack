use std::sync::Arc;

use rspack_core::{Chunk, Module};
use rspack_error::Result;

pub struct ChunkNameGetterFnCtx<'a> {
  pub module: &'a dyn Module,
  pub chunks: &'a Vec<&'a Chunk>,
  pub cache_group_key: &'a str,
}

type ChunkNameGetterFn =
  Arc<dyn for<'a> Fn(ChunkNameGetterFnCtx<'a>) -> Result<Option<String>> + Send + Sync>;

#[derive(Clone)]
pub enum ChunkNameGetter {
  String(String),
  Fn(ChunkNameGetterFn),
  Disabled,
}
