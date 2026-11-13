use rspack_cacheable::cacheable;

#[cacheable]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoaderRunnerOptions {
  pub cache: bool,
  /// Loader name used as part of the cache key.
  pub loader_name: String,
  /// Stable serialization of the loader options used as part of the etag.
  pub options_cache_key: String,
  /// JavaScript loader context source map override.
  pub source_map: Option<bool>,
  /// Loader implementation version or file hash used as part of the etag.
  pub loader_version: String,
}
