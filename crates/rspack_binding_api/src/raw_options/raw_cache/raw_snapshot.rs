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
  pub immutable_paths: Vec<RawPathMatcher>,
  #[napi(ts_type = r#"Array<string|RegExp>"#)]
  pub unmanaged_paths: Vec<RawPathMatcher>,
  #[napi(ts_type = r#"Array<string|RegExp>"#)]
  pub managed_paths: Vec<RawPathMatcher>,
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
    let hash = value.hash.unwrap_or_default();
    let timestamp = if hash {
      value.timestamp.unwrap_or_default()
    } else {
      true
    };
    SnapshotStrategyOptions::new(hash, timestamp)
  }
}

impl From<RawSnapshotOptions> for SnapshotOptions {
  fn from(value: RawSnapshotOptions) -> Self {
    let options = SnapshotOptions::new(
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

    if let Some(context_module) = value.context_module {
      options.with_context_module_strategy(context_module.into())
    } else {
      options
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{RawSnapshotOptions, RawSnapshotStrategyOptions, SnapshotStrategyOptions};

  fn to_strategy(hash: Option<bool>, timestamp: Option<bool>) -> SnapshotStrategyOptions {
    RawSnapshotStrategyOptions { hash, timestamp }.into()
  }

  #[test]
  fn should_align_context_module_strategy_defaults_with_webpack() {
    let strategy = to_strategy(None, None);
    assert!(!strategy.hash);
    assert!(strategy.timestamp);

    let strategy = to_strategy(Some(false), Some(false));
    assert!(!strategy.hash);
    assert!(strategy.timestamp);

    let strategy = to_strategy(Some(true), None);
    assert!(strategy.hash);
    assert!(!strategy.timestamp);

    let strategy = to_strategy(Some(true), Some(true));
    assert!(strategy.hash);
    assert!(strategy.timestamp);
  }

  #[test]
  fn should_use_default_context_module_strategy_when_omitted() {
    let options: rspack_core::cache::persistent::snapshot::SnapshotOptions =
      RawSnapshotOptions::default().into();
    let strategy = options.context_module_strategy();
    assert!(!strategy.hash);
    assert!(strategy.timestamp);
  }
}
