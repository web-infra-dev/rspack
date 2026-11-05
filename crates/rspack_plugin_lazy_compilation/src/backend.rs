use std::{future::Future, sync::Arc};

use rspack_collections::IdentifierSet;
use rspack_error::Result;

pub trait Backend: std::fmt::Debug + Send + Sync {
  fn collect_reserved_externals(
    &mut self,
    layer: Option<String>,
  ) -> impl Future<Output = Result<Arc<[String]>>> + Send + '_;

  fn take_newly_active_modules(
    &mut self,
  ) -> impl Future<Output = Result<IdentifierSet>> + Send + '_;
}
