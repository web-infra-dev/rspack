use super::Cache;

/// Cache implementation used when build caching is disabled.
#[derive(Debug)]
pub struct DisableCache;

#[async_trait::async_trait]
impl Cache for DisableCache {}
