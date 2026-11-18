use std::{borrow::Cow, fs, sync::Arc};

use rspack_loader_runner::DescriptionData;
use rspack_resolver::Resolver;
use serde_json::json;

fn fixture(manifest: &str) -> tempfile::TempDir {
  let dir = tempfile::tempdir().expect("create fixture");
  fs::write(dir.path().join("package.json"), manifest).expect("write package.json");
  fs::write(dir.path().join("index.js"), "export default 1;").expect("write entry");
  dir
}

#[tokio::test]
async fn description_data_releases_resolver_generations() {
  let dir = fixture("{}");
  let resolver = Resolver::default();
  let mut descriptions = Vec::new();
  for generation in 0..8 {
    let manifest = json!({
      "name": "fixture",
      "custom": { "generation": generation },
      "description": "ignored".repeat(64 * 1024),
      "scripts": { "build": "x".repeat(64 * 1024) }
    });
    fs::write(dir.path().join("package.json"), manifest.to_string()).expect("write manifest");
    let resolution = resolver
      .resolve(dir.path(), "./index.js")
      .await
      .expect("resolve entry");
    let package = resolution.package_json().expect("package metadata");
    let weak = Arc::downgrade(package);
    descriptions.push(DescriptionData::from_package_json(package));
    drop(resolution);
    resolver.clear_cache();
    assert!(
      weak.upgrade().is_none(),
      "module metadata must not retain the resolver package"
    );
  }
  for (generation, description) in descriptions.iter().enumerate() {
    assert_eq!(
      description.get(["custom", "generation"]).as_deref(),
      Some(&json!(generation))
    );
    assert_eq!(
      description.json(),
      &json!({"name": "fixture", "custom": {"generation": generation}})
    );
  }
}

#[tokio::test]
async fn description_fields_are_reused_and_survive_cache_serialization() {
  let manifest = json!({
    "name": "fixture",
    "sideEffects": (0..1024).map(|i| format!("./file-{i}.js")).collect::<Vec<_>>(),
    "custom": {
      "null": null,
      "array": [1, "é", "a\nb"],
      "float": 1.000_000_000_000_000_2,
      "numbers": [1.0, -0.0, u64::MAX, i64::MIN]
    },
    "unread": { "key": "preserved" }
  });
  let dir = fixture(&manifest.to_string());
  let resolver = Resolver::default();
  let resolution = resolver
    .resolve(dir.path(), "./index.js")
    .await
    .expect("resolve entry");
  let package = resolution.package_json().expect("package metadata");
  let description = DescriptionData::from_package_json(package);
  let other = DescriptionData::from_package_json(package);
  let first = description.get(["sideEffects"]).expect("sideEffects");
  assert!(matches!(first, Cow::Borrowed(_)));
  for _ in 0..100 {
    let next = other.get(["sideEffects"]).expect("sideEffects");
    assert!(
      std::ptr::eq(first.as_ref(), next.as_ref()),
      "reuse the same parsed field across modules"
    );
  }
  assert_eq!(
    description.get(["custom", "float"]).as_deref(),
    Some(&manifest["custom"]["float"])
  );
  assert!(description.get(["custom", "array", "0"]).is_none());
  assert!(description.get(["missing"]).is_none());
  let bytes = rspack_cacheable::to_bytes(&description, &()).expect("serialize description");
  let restored: DescriptionData =
    rspack_cacheable::from_bytes(&bytes, &()).expect("restore description");
  assert_eq!(restored.json(), &manifest);
  assert_eq!(description.json(), &manifest);
  // Previously returned references stay valid after the complete mirror is initialized.
  assert_eq!(first.as_ref(), &manifest["sideEffects"]);
  assert_eq!(
    restored.get(["custom", "null"]).as_deref(),
    Some(&json!(null))
  );
}

#[tokio::test]
async fn empty_description_path_is_independent_of_materialization() {
  for manifest in ["[]", "42", "null", "\"text\"", "{\"custom\":1}"] {
    let dir = fixture(manifest);
    let resolver = Resolver::default();
    let resolution = resolver
      .resolve(dir.path(), "./index.js")
      .await
      .expect("resolve entry");
    let package = resolution.package_json().expect("package metadata");
    let description = DescriptionData::from_package_json(package);
    let expected = if manifest.starts_with('{') {
      json!({"custom": 1})
    } else {
      json!(null)
    };
    assert_eq!(package.raw_json_at([]).as_deref(), Some(&expected));
    assert_eq!(description.get([]).as_deref(), Some(&expected));
    assert_eq!(package.raw_json().as_ref(), &expected);
    assert_eq!(package.raw_json_at([]).as_deref(), Some(&expected));
    let bytes = rspack_cacheable::to_bytes(&description, &()).expect("serialize description");
    let restored: DescriptionData =
      rspack_cacheable::from_bytes(&bytes, &()).expect("restore description");
    assert_eq!(restored.get([]).as_deref(), Some(&expected));
  }
}
