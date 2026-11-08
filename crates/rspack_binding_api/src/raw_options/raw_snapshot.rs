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

#[derive(Debug)]
#[napi(object)]
pub struct RawSnapshotOptions {
  #[napi(ts_type = r#"Array<string|RegExp>"#)]
  pub immutable_paths: Vec<RawPathMatcher>,
  #[napi(ts_type = r#"Array<string|RegExp>"#)]
  pub unmanaged_paths: Vec<RawPathMatcher>,
  #[napi(ts_type = r#"Array<string|RegExp>"#)]
  pub managed_paths: Vec<RawPathMatcher>,
  pub build_dependencies: RawSnapshotStrategyOptions,
  pub resolve_build_dependencies: RawSnapshotStrategyOptions,
  pub module: RawSnapshotStrategyOptions,
  pub context_module: RawSnapshotStrategyOptions,
  pub resolve: RawSnapshotStrategyOptions,
}

impl From<RawSnapshotOptions> for SnapshotOptions {
  fn from(value: RawSnapshotOptions) -> Self {
    Self {
      immutable_paths: value
        .immutable_paths
        .into_iter()
        .map(normalize_raw_path_matcher)
        .collect(),
      unmanaged_paths: value
        .unmanaged_paths
        .into_iter()
        .map(normalize_raw_path_matcher)
        .collect(),
      managed_paths: value
        .managed_paths
        .into_iter()
        .map(normalize_raw_path_matcher)
        .collect(),
      build_dependencies: value.build_dependencies.into(),
      resolve_build_dependencies: value.resolve_build_dependencies.into(),
      module: value.module.into(),
      context_module: value.context_module.into(),
      resolve: value.resolve.into(),
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawSnapshotStrategyOptions {
  pub hash: bool,
  pub timestamp: bool,
}

impl From<RawSnapshotStrategyOptions> for SnapshotStrategyOptions {
  fn from(value: RawSnapshotStrategyOptions) -> Self {
    Self::new(value.hash, value.timestamp)
  }
}
