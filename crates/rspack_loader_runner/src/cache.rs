use rspack_cacheable::cacheable;

#[cacheable]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoaderRunnerOptions {
  pub cache: bool,
  /// Loader name used as part of the cache key.
  pub loader_name: String,
  /// Stable serialization of the loader options used as part of the etag.
  pub options_cache_key: String,
  /// Loader implementation version or file hash used as part of the etag.
  pub loader_version: String,
  /// A portable build couldn't compare the options, so the chain skips the loader result cache.
  pub incomparable_options: bool,
}
