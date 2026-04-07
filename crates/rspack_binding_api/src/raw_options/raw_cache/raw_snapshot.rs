use napi::Either;
use napi_derive::napi;
use rspack_core::cache::persistent::snapshot::{PathMatcher, SnapshotOptions};

use crate::js_regex::JsRegExp;

type RawPathMatcher = Either<String, JsRegExp>;

fn normalize_raw_path_matcher(value: RawPathMatcher) -> rspack_error::Result<PathMatcher> {
  Ok(match value {
    Either::A(s) => PathMatcher::String(s),
    Either::B(reg) => PathMatcher::Regexp(reg.try_into()?),
  })
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

impl TryFrom<RawSnapshotOptions> for SnapshotOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawSnapshotOptions) -> Result<Self, Self::Error> {
    Ok(SnapshotOptions::new(
      value
        .immutable_paths
        .into_iter()
        .map(normalize_raw_path_matcher)
        .collect::<Result<_, _>>()?,
      value
        .unmanaged_paths
        .into_iter()
        .map(normalize_raw_path_matcher)
        .collect::<Result<_, _>>()?,
      value
        .managed_paths
        .into_iter()
        .map(normalize_raw_path_matcher)
        .collect::<Result<_, _>>()?,
    ))
  }
}
