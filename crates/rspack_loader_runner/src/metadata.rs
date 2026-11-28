use std::sync::Arc;

use rspack_cacheable::cacheable;

use crate::LoaderFilter;

#[cacheable]
#[derive(Debug, Default)]
pub struct LoaderMetadata {
  pub cache: bool,
  pub options_cache_key: String,
  pub filter: Option<LoaderFilter>,
}

/// Shared loader metadata aligned with the loaders in the chain.
pub type LoaderMetadataList = Box<[Option<Arc<LoaderMetadata>>]>;
