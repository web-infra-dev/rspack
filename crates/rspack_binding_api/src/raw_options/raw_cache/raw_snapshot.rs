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
  pub dependencies: Option<RawSnapshotStrategyOptions>,
  pub context_dependencies: Option<RawSnapshotStrategyOptions>,
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

    let options = if let Some(dependencies) = value.dependencies {
      options.with_dependencies_strategy(dependencies.into())
    } else {
      options
    };

    if let Some(context_dependencies) = value.context_dependencies {
      return options.with_context_dependencies_strategy(context_dependencies.into());
    }

    options
  }
}

#[cfg(test)]
mod tests {
  use super::{RawSnapshotOptions, RawSnapshotStrategyOptions, SnapshotStrategyOptions};

  fn to_strategy(hash: Option<bool>, timestamp: Option<bool>) -> SnapshotStrategyOptions {
    RawSnapshotStrategyOptions { hash, timestamp }.into()
  }

  #[test]
  fn should_align_snapshot_strategy_defaults_with_webpack() {
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
  fn should_use_default_snapshot_strategies_when_omitted() {
    let options: rspack_core::cache::persistent::snapshot::SnapshotOptions =
      RawSnapshotOptions::default().into();
    let dependencies_strategy = options.dependencies_strategy();
    assert!(!dependencies_strategy.hash);
    assert!(dependencies_strategy.timestamp);

    let context_dependencies_strategy = options.context_dependencies_strategy();
    assert!(!context_dependencies_strategy.hash);
    assert!(context_dependencies_strategy.timestamp);
  }

  #[test]
  fn should_apply_raw_dependencies_strategy() {
    let options: rspack_core::cache::persistent::snapshot::SnapshotOptions = RawSnapshotOptions {
      dependencies: Some(RawSnapshotStrategyOptions {
        hash: Some(true),
        timestamp: Some(true),
      }),
      ..Default::default()
    }
    .into();
    let strategy = options.dependencies_strategy();
    assert!(strategy.hash);
    assert!(strategy.timestamp);
  }
}
