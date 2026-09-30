use std::collections::HashSet;

use rspack_collections::Identifiable;
use rspack_core::{Context, runtime_mode::RuntimeMode};
use rspack_plugin_mf::{
  ConsumeSharedModule, ConsumeVersion, EnhancedConsumeOptions, ProvideSharedModule, ProvideVersion,
  ShareScope,
};

#[test]
fn provider_identifiers_distinguish_runtime_behavior() {
  let options = [
    (false, None, None, None, None),
    (true, None, None, None, None),
    (false, Some(true), None, None, None),
    (false, Some(false), None, None, None),
    (false, None, Some(ConsumeVersion::False), None, None),
    (
      false,
      None,
      Some(ConsumeVersion::Version("^1.0.0".into())),
      None,
      None,
    ),
    (false, None, None, Some(true), None),
    (false, None, None, Some(false), None),
    (false, None, None, None, Some("runtime-infer".to_string())),
  ];
  let mut identifiers = HashSet::new();
  for (eager, singleton, required_version, strict_version, tree_shaking_mode) in options {
    let module = ProvideSharedModule::new_enhanced(
      ShareScope::Single("default".into()),
      "react".into(),
      ProvideVersion::Version("1.0.0".into()),
      "/shared.js".into(),
      eager,
      singleton,
      required_version,
      strict_version,
      None,
      tree_shaking_mode,
      RuntimeMode::Webpack,
    );
    assert!(
      identifiers.insert(module.identifier()),
      "provider behavior must not collapse"
    );
  }
}

fn identifiers(scope: ShareScope, key: &str, layer: Option<&str>) -> [String; 2] {
  let consume = ConsumeSharedModule::new_enhanced(
    Context::from(""),
    EnhancedConsumeOptions {
      request: None,
      issuer_layer: None,
      layer: layer.map(str::to_string),
      import: None,
      import_resolved: Some("/shared.js".to_string()),
      share_key: key.to_string(),
      share_scope: scope.clone(),
      required_version: None,
      package_name: None,
      strict_version: true,
      singleton: false,
      eager: false,
      tree_shaking_mode: None,
    },
    RuntimeMode::Webpack,
  );
  let provide = ProvideSharedModule::new_enhanced(
    scope,
    key.to_string(),
    ProvideVersion::Version("1.0.0".to_string()),
    "/shared.js".to_string(),
    false,
    None,
    None,
    None,
    layer.map(str::to_string),
    None,
    RuntimeMode::Webpack,
  );
  [
    consume.identifier().to_string(),
    provide.identifier().to_string(),
  ]
}

#[test]
fn test_identifiers_preserve_webpack_prefixes() {
  let [consume, provide] = identifiers(ShareScope::Single("default".into()), "react", None);
  assert!(consume.starts_with(
    "consume shared module (default) react@* (strict) (fallback: /shared.js) [identity:"
  ));
  assert!(
    provide.starts_with("provide shared module (default) react@1.0.0 = /shared.js [identity:")
  );
  assert_eq!(consume.split(' ').nth(4), Some("react@*"));
  assert_eq!(provide.split(' ').nth(4), Some("react@1.0.0"));
  let [consume, provide] = identifiers(
    ShareScope::Single("default".into()),
    "react",
    Some("server"),
  );
  assert!(consume.starts_with("consume shared module (default) (server) react@* "));
  assert!(provide.starts_with("provide shared module (default) (server) react@1.0.0 = "));
}

#[test]
fn test_identifiers_distinguish_scope_and_layer_delimiters() {
  let cases = [
    (ShareScope::Single("a|b".into()), "pkg", None),
    (
      ShareScope::Multiple(vec!["a".into(), "b".into()]),
      "pkg",
      None,
    ),
    (
      ShareScope::Multiple(vec!["b".into(), "a".into()]),
      "pkg",
      None,
    ),
    (
      ShareScope::Multiple(vec!["a:".into(), "b".into()]),
      "pkg",
      None,
    ),
    (
      ShareScope::Multiple(vec!["a".into(), ":b".into()]),
      "pkg",
      None,
    ),
    (ShareScope::Single("default".into()), "c", Some("a) b")),
    (ShareScope::Single("default".into()), "b) c", Some("a")),
    (ShareScope::Single("default".into()), "(a) b) c", None),
    (ShareScope::Single("default".into()), "pkg", Some("")),
    (ShareScope::Single("default".into()), "pkg", None),
  ];
  let mut consumes = HashSet::new();
  let mut provides = HashSet::new();
  for (scope, key, layer) in cases {
    let [consume, provide] = identifiers(scope, key, layer);
    assert!(consumes.insert(consume));
    assert!(provides.insert(provide));
  }
}
