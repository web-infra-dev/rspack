use napi::Either;
use napi_derive::napi;
use rspack_core::cache::persistent::snapshot::{PathMatcher, SnapshotOptions};
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
}

impl From<RawSnapshotOptions> for SnapshotOptions {
  fn from(value: RawSnapshotOptions) -> Self {
    SnapshotOptions::new(
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
    )
  }
}
