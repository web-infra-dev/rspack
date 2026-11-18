#![cfg(feature = "yarn_pnp")]

use std::{fs, path::Path};

use cow_utils::CowUtils;
use rspack_resolver::{ResolveOptions, Resolver};

#[tokio::test]
async fn resolve_pnp_dependency_from_global_cache_after_clear_cache() {
  let fixture = dunce::canonicalize(
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/pnp-global-cache-rebuild"),
  )
  .unwrap();
  let resolver = Resolver::new(ResolveOptions {
    extensions: vec![".js".into()],
    enable_pnp: true,
    ..ResolveOptions::default()
  });

  let a = resolver
    .resolve(fixture.join("project"), "a")
    .await
    .unwrap();
  assert_eq!(
    a.path(),
    fixture.join("cache/a.zip/node_modules/a/index.js")
  );
  let issuer = a.path().parent().unwrap();
  let expected = fixture.join("cache/b.zip/node_modules/b/index.js");
  assert_eq!(
    resolver.resolve(issuer, "b").await.unwrap().path(),
    expected
  );

  resolver.clear_cache();

  assert_eq!(
    resolver.resolve(issuer, "b").await.unwrap().path(),
    expected
  );
}

#[tokio::test]
async fn reload_pnp_manifest_after_clear_cache() {
  let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/pnp-global-cache-rebuild");
  let temp = tempfile::tempdir().unwrap();
  let root = dunce::canonicalize(temp.path()).unwrap();
  let project = root.join("project");
  let cache = root.join("cache");
  fs::create_dir(&project).unwrap();
  fs::create_dir(&cache).unwrap();
  for name in ["a.zip", "b.zip"] {
    fs::copy(fixture.join("cache").join(name), cache.join(name)).unwrap();
  }
  fs::copy(cache.join("b.zip"), cache.join("b-v2.zip")).unwrap();
  let manifest_path = project.join(".pnp.cjs");
  let manifest = fs::read_to_string(fixture.join("project/.pnp.cjs")).unwrap();
  fs::write(&manifest_path, &manifest).unwrap();

  let options = ResolveOptions {
    extensions: vec![".js".into()],
    enable_pnp: true,
    ..ResolveOptions::default()
  };
  let resolver = Resolver::new(options.clone());
  let cloned = resolver.clone_with_options(options);
  let a = resolver.resolve(&project, "a").await.unwrap();
  let issuer = a.path().parent().unwrap();
  assert_eq!(
    cloned.resolve(issuer, "b").await.unwrap().path(),
    cache.join("b.zip/node_modules/b/index.js")
  );

  let updated_manifest = manifest.cow_replace("../cache/b.zip/", "../cache/b-v2.zip/");
  fs::write(&manifest_path, updated_manifest.as_bytes()).unwrap();
  resolver.clear_cache();
  let expected = cache.join("b-v2.zip/node_modules/b/index.js");
  assert_eq!(cloned.resolve(issuer, "b").await.unwrap().path(), expected);

  fs::remove_file(&manifest_path).unwrap();
  cloned.clear_cache();
  assert!(resolver.resolve(issuer, "b").await.is_err());

  fs::write(&manifest_path, updated_manifest.as_bytes()).unwrap();
  resolver.clear_cache();
  assert_eq!(cloned.resolve(issuer, "b").await.unwrap().path(), expected);
}
