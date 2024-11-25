use super::Cache;

/// Memory cache implementation
///
/// Since the compilation will keep all the data from the last compilation,
/// the memory cache does nothing
#[derive(Debug)]
pub struct MemoryCache;

impl Cache for MemoryCache {}
