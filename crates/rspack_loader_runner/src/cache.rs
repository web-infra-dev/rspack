use rspack_cacheable::{cacheable, with::Skip};

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
  pub parallel: bool,
  /// Handle for ordinary JavaScript loader options kept in the main JS isolate.
  #[cacheable(with=Skip)]
  pub js_options_handle: Option<u32>,
  /// Rule-set reference used to preserve the public loader query (`??ident`) after resolving it.
  pub ident: Option<String>,
}
