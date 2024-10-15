use std::sync::Arc;

use rspack_cacheable::{cacheable, with::AsConverter, DeserializeError, SerializeError};

use super::CacheContext;
use crate::CompilerOptions;

#[cacheable]
pub struct FromContext;

impl AsConverter<Arc<CompilerOptions>> for FromContext {
  type Context = CacheContext;
  fn serialize(_data: &Arc<CompilerOptions>, _ctx: &Self::Context) -> Result<Self, SerializeError> {
    Ok(FromContext)
  }
  fn deserialize(&self, ctx: &Self::Context) -> Result<Arc<CompilerOptions>, DeserializeError> {
    Ok(ctx.options.clone())
  }
}
