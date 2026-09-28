#[cfg(allocative)]
use rspack_util::allocative;

use super::Cache;

/// Cache implementation used when build caching is disabled.
#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct DisableCache;

#[async_trait::async_trait]
impl Cache for DisableCache {}
