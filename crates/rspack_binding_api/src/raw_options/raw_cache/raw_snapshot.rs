use napi::Either;
use napi_derive::napi;
use rspack_core::{PathMatcher, SnapshotOptions, SnapshotStrategyOptions};
use rspack_regex::RspackRegex;

type RawPathMatcher = Either<String, RspackRegex>;

fn normalize_raw_path_matcher(value: RawPathMatcher) -> PathMatcher {
  match value {
    Either::A(s) => PathMatcher::String(s),
    Either::B(reg) => PathMatcher::Regexp(reg),
  }
}

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawSnapshotOptions {
  #[napi(ts_type = r#"Array<string|RegExp>"#)]
  pub immutable_paths: Vec<RawPathMatcher>,
  #[napi(ts_type = r#"Array<string|RegExp>"#)]
  pub unmanaged_paths: Vec<RawPathMatcher>,
  #[napi(ts_type = r#"Array<string|RegExp>"#)]
  pub managed_paths: Vec<RawPathMatcher>,
  pub module: Option<RawSnapshotStrategy>,
  pub context_module: Option<RawSnapshotStrategy>,
  pub resolve: Option<RawSnapshotStrategy>,
  pub build_dependencies: Option<RawSnapshotStrategy>,
  pub resolve_build_dependencies: Option<RawSnapshotStrategy>,
}

#[derive(Debug)]
#[napi(object)]
pub struct RawSnapshotStrategy {
  pub timestamp: Option<bool>,
  pub hash: Option<bool>,
}

impl From<RawSnapshotStrategy> for SnapshotStrategyOptions {
  fn from(value: RawSnapshotStrategy) -> Self {
    Self::new(
      value.hash.unwrap_or(false),
      value.timestamp.unwrap_or(false),
    )
  }
}

impl From<RawSnapshotOptions> for SnapshotOptions {
  fn from(value: RawSnapshotOptions) -> Self {
    let mut options = SnapshotOptions::new(
      value
        .immutable_paths
        .into_iter()
        .map(normalize_raw_path_matcher)
        .collect(),
      value
        .unmanaged_paths
        .into_iter()
        .map(normalize_raw_path_matcher)
        .collect(),
      value
        .managed_paths
        .into_iter()
        .map(normalize_raw_path_matcher)
        .collect(),
    );
    options.module = value.module.map_or(options.module, Into::into);
    options.context_module = value
      .context_module
      .map_or(options.context_module, Into::into);
    options.resolve = value.resolve.map_or(options.resolve, Into::into);
    options.build_dependencies = value
      .build_dependencies
      .map_or(options.build_dependencies, Into::into);
    options.resolve_build_dependencies = value
      .resolve_build_dependencies
      .map_or(options.resolve_build_dependencies, Into::into);
    options
  }
}
