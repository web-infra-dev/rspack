use rspack_cacheable::cacheable;

#[cacheable]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoaderRunnerOptions {
  pub cache: bool,
  /// Loader name, stable options and loader version/file hash.
  pub cache_key: String,
}
