use std::future::Future;

use rspack_collections::IdentifierSet;
use rspack_error::Result;

pub trait Backend: std::fmt::Debug + Send + Sync {
  fn current_active_modules(&mut self) -> impl Future<Output = Result<IdentifierSet>> + Send + '_;
}
