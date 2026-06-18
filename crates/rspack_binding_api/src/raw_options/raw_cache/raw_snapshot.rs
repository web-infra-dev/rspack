use napi::Either;
use napi_derive::napi;
use rspack_core::cache::persistent::snapshot::{
  PathMatcher, SnapshotOptions, SnapshotStrategyOptions,
};
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
  pub immutable_paths: Option<Vec<RawPathMatcher>>,
  #[napi(ts_type = r#"Array<string|RegExp>"#)]
  pub unmanaged_paths: Option<Vec<RawPathMatcher>>,
  #[napi(ts_type = r#"Array<string|RegExp>"#)]
  pub managed_paths: Option<Vec<RawPathMatcher>>,
  pub context_module: Option<RawSnapshotStrategyOptions>,
}

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawSnapshotStrategyOptions {
  pub hash: Option<bool>,
  pub timestamp: Option<bool>,
}

impl From<RawSnapshotStrategyOptions> for SnapshotStrategyOptions {
  fn from(value: RawSnapshotStrategyOptions) -> Self {
    SnapshotStrategyOptions::new(
      value.hash.unwrap_or_default(),
      value.timestamp.unwrap_or_default(),
    )
  }
}

impl From<RawSnapshotOptions> for SnapshotOptions {
  fn from(value: RawSnapshotOptions) -> Self {
    let options = SnapshotOptions::new(
      value
        .immutable_paths
        .unwrap_or_default()
        .into_iter()
        .map(normalize_raw_path_matcher)
        .collect(),
      value
        .unmanaged_paths
        .unwrap_or_default()
        .into_iter()
        .map(normalize_raw_path_matcher)
        .collect(),
      value
        .managed_paths
        .unwrap_or_default()
        .into_iter()
        .map(normalize_raw_path_matcher)
        .collect(),
    );

    if let Some(context_module) = value.context_module {
      options.with_context_module_strategy(context_module.into())
    } else {
      options
    }
  }
}
