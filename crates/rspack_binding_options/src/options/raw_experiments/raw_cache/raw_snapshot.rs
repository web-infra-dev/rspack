use napi::Either;
use napi_derive::napi;
use rspack_core::cache::persistent::snapshot::{PathMatcher, SnapshotOptions};
use rspack_regex::RspackRegex;

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawSnapshotOptions {
  pub immutable_paths: Vec<RawPathMatcher>,
  pub unmanaged_paths: Vec<RawPathMatcher>,
  pub managed_paths: Vec<RawPathMatcher>,
}

type RawPathMatcher = Either<String, RspackRegex>;

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

fn normalize_raw_path_matcher(value: RawPathMatcher) -> PathMatcher {
  match value {
    Either::A(s) => PathMatcher::String(s),
    Either::B(reg) => PathMatcher::Regexp(reg),
  }
}
