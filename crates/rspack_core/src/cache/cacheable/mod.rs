mod from_context;

use std::sync::Arc;

pub use from_context::FromContext;

use crate::CompilerOptions;

#[derive(Debug)]
pub struct CacheContext {
  pub options: Arc<CompilerOptions>,
}

pub type ArcCacheContext = Arc<CacheContext>;
