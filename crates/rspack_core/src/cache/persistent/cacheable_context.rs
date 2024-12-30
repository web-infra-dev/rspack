use std::{any::Any, sync::Arc};

use rspack_cacheable::{cacheable, with::AsConverter, DeserializeError, SerializeError};
use rspack_fs::ReadableFileSystem;

use crate::CompilerOptions;

#[derive(Debug)]
pub struct CacheableContext {
  pub options: Arc<CompilerOptions>,
  pub input_filesystem: Arc<dyn ReadableFileSystem>,
}

#[cacheable]
pub struct FromContext;

impl AsConverter<Arc<CompilerOptions>> for FromContext {
  fn serialize(_data: &Arc<CompilerOptions>, _ctx: &dyn Any) -> Result<Self, SerializeError> {
    Ok(FromContext)
  }
  fn deserialize(self, ctx: &dyn Any) -> Result<Arc<CompilerOptions>, DeserializeError> {
    let Some(ctx) = ctx.downcast_ref::<CacheableContext>() else {
      return Err(DeserializeError::NoContext);
    };
    Ok(ctx.options.clone())
  }
}
