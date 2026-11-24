use rspack_cacheable::cacheable;
use rspack_paths::InternedPath;

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
  /// Additional cache identifier configured on the loader rule.
  pub cache_identifier: Option<String>,
  /// Additional build dependency paths configured on the loader rule.
  pub cache_build_dependencies: Vec<InternedPath>,
}
