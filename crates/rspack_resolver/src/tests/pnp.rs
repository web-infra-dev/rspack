//! Not part of enhanced_resolve's test suite
//!
//! enhanced_resolve's test <https://github.com/webpack/enhanced-resolve/blob/main/test/pnp.test.js>
//! cannot be ported over because it uses mocks on `pnpApi` provided by the runtime.

use camino::Utf8Path;
use cow_utils::CowUtils;

use crate::{
  ArcPath, ResolveContext, ResolveError::NotFound, ResolveOptions, Resolver, path::PathUtil,
};

#[tokio::test]
async fn pnp1() {
  let fixture = super::fixture_root().join("pnp");

  let resolver = Resolver::new(ResolveOptions {
    extensions: vec![".js".into()],
    condition_names: vec!["import".into()],
    ..ResolveOptions::default()
  });

  assert_eq!(
    resolver
      .resolve(&fixture, "is-even")
      .await
      .map(|r| r.full_path()),
    Ok(fixture.join(
      ".yarn/cache/is-even-npm-1.0.0-9f726520dc-2728cc2f39.zip/node_modules/is-even/index.js"
    ))
  );

  assert_eq!(
    resolver
      .resolve(&fixture, "lodash.zip")
      .await
      .map(|r| r.full_path()),
    Ok(fixture.join(
      ".yarn/cache/lodash.zip-npm-4.2.0-5299417ec8-e596da80a6.zip/node_modules/lodash.zip/index.js"
    ))
  );

  assert_eq!(
    resolver
      .resolve(
        fixture
          .join(".yarn/cache/is-even-npm-1.0.0-9f726520dc-2728cc2f39.zip/node_modules/is-even"),
        "is-odd"
      )
      .await
      .map(|r| r.full_path()),
    Ok(
      fixture.join(
        ".yarn/cache/is-odd-npm-0.1.2-9d980a9da8-7dc6c6fd00.zip/node_modules/is-odd/index.js"
      )
    ),
  );

  assert_eq!(
    resolver
      .resolve(&fixture, "is-odd")
      .await
      .map(|r| r.full_path()),
    Ok(
      fixture.join(
        ".yarn/cache/is-odd-npm-3.0.1-93c3c3f41b-89ee2e353c.zip/node_modules/is-odd/index.js"
      )
    ),
  );

  assert_eq!(
    resolver
      .resolve(&fixture, "preact")
      .await
      .map(|r| r.full_path()),
    Ok(fixture.join(
      ".yarn/cache/preact-npm-10.25.4-2dd2c0aa44-33a009d614.zip/node_modules/preact/dist/preact.mjs"
    )),
  );

  assert_eq!(
        resolver.resolve(&fixture, "preact/devtools").await.map(|r| r.full_path()),
        Ok(fixture.join(
            ".yarn/cache/preact-npm-10.25.4-2dd2c0aa44-33a009d614.zip/node_modules/preact/devtools/dist/devtools.mjs"
        )),
    );
}

#[tokio::test]
async fn pnp_file_dependencies() {
  let fixture = super::fixture_root().join("pnp");

  let resolver = Resolver::new(ResolveOptions {
    extensions: vec![".js".into()],
    condition_names: vec!["import".into()],
    ..ResolveOptions::default()
  });

  let mut ctx = ResolveContext::default();
  let result = resolver
    .resolve_with_context(&fixture, "is-even", &mut ctx)
    .await;
  assert!(result.is_ok());
  assert!(
    ctx
      .file_dependencies
      .contains(&ArcPath::from(fixture.join(".pnp.cjs"))),
    ".pnp.cjs should be in file_dependencies, got: {:?}",
    ctx.file_dependencies
  );
}

// https://github.com/webpack/enhanced-resolve/blob/main/lib/PnpPlugin.js#L118
// `fullySpecified` should be disabled for bare specifiers without subpath,
// so that the `main` field value (e.g. "index" without extension) can be resolved.
#[tokio::test]
async fn pnp_bare_specifier_fully_specified() {
  let fixture = super::fixture_root().join("pnp");

  let resolver = Resolver::new(ResolveOptions {
    extensions: vec![".js".into()],
    condition_names: vec!["node".into(), "import".into()],
    fully_specified: true,
    ..ResolveOptions::default()
  });

  // `extend` has "main": "index" (no extension) and no "exports" field.
  assert!(
    resolver.resolve(&fixture, "extend").await.is_ok(),
    "should resolve successfully without fully specified check for bare specifier with main field without extension"
  );
}

#[tokio::test]
async fn pnp_resolve_description_file() {
  let fixture = super::fixture_root().join("pnp");

  let resolver = Resolver::new(ResolveOptions {
    extensions: vec![".js".into()],
    condition_names: vec!["import".into()],
    ..ResolveOptions::default()
  });

  let full_path = fixture
    .join(
      ".yarn/cache/preact-npm-10.25.4-2dd2c0aa44-33a009d614.zip/node_modules/preact/dist/preact.js",
    )
    .to_str()
    .expect("path should be UTF-8")
    .to_string();

  let r = resolver.resolve(&fixture, &full_path).await.unwrap();

  assert_eq!(
    r.package_json
      .unwrap()
      .path
      .to_str()
      .expect("path should be UTF-8")
      .to_string(),
    Utf8Path::from_path(&fixture)
      .expect("path should be UTF-8")
      .join(".yarn/cache/preact-npm-10.25.4-2dd2c0aa44-33a009d614.zip/node_modules/preact")
      .join("package.json")
      .normalize()
      .to_string()
  );
}

#[tokio::test]
async fn resolve_in_pnp_linked_folder() {
  let fixture = super::fixture_root().join("pnp");

  let resolver = Resolver::new(ResolveOptions {
    extensions: vec![".js".into()],
    condition_names: vec!["import".into()],
    ..ResolveOptions::default()
  });

  assert_eq!(
    resolver
      .resolve(&fixture, "lib/lib.js")
      .await
      .map(|r| r.full_path()),
    Ok(fixture.join("shared/lib.js"))
  );
}

#[tokio::test]
async fn resolve_pnp_pkg_should_failed_while_disable_pnp_mode() {
  let fixture = super::fixture_root().join("pnp");

  let resolver = Resolver::new(ResolveOptions {
    enable_pnp: false,
    ..ResolveOptions::default()
  });

  assert_eq!(
    resolver
      .resolve(&fixture, "is-even")
      .await
      .map(|r| r.full_path()),
    Err(NotFound("is-even".to_string()))
  );
}

#[cfg(target_os = "windows")]
#[tokio::test]
async fn resolve_pnp_with_global_cache_enabled_windows() {
  let fixture = super::fixture_root().join("pnp-global-cache-enabled");

  let resolver = Resolver::new(ResolveOptions {
    extensions: vec![".js".into()],
    enable_pnp: true,
    ..ResolveOptions::default()
  });

  let resolved = resolver
    .resolve(&fixture, "path-to-regexp")
    .await
    .map(|r| r.full_path())
    .unwrap();

  let module_root = resolved.parent().unwrap();
  let module_root_str = module_root
    .to_str()
    .expect("path should be UTF-8")
    .replace('\\', "/");
  assert!(
    module_root_str.contains("/Yarn/Berry/cache/path-to-regexp"),
    "Expected global cache path, got: {module_root_str}"
  );

  let resolved_from_cache = resolver
    .resolve(module_root, "./index.js")
    .await
    .map(|r| {
      r.full_path()
        .to_str()
        .expect("path should be UTF-8")
        .replace('\\', "/")
        .to_string()
    })
    .unwrap();
  assert!(
    resolved_from_cache.contains("/Yarn/Berry/cache/path-to-regexp"),
    "Expected global cache path, got: {resolved_from_cache}"
  );
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn resolve_pnp_with_global_cache_enabled_unix() {
  let fixture = super::fixture_root().join("pnp-global-cache-enabled");

  let resolver = Resolver::new(ResolveOptions {
    extensions: vec![".js".into()],
    enable_pnp: true,
    ..ResolveOptions::default()
  });

  let resolved = resolver
    .resolve(&fixture, "path-to-regexp")
    .await
    .map(|r| r.full_path())
    .unwrap();

  let module_root = resolved.parent().unwrap();
  let module_root_str = module_root.to_str().expect("path should be UTF-8");
  assert!(
    module_root_str.contains("/.yarn/berry/cache/path-to-regexp"),
    "Expected global cache path, got: {module_root_str}"
  );

  let resolved_from_cache = resolver
    .resolve(module_root, "./index.js")
    .await
    .map(|r| r.full_path())
    .unwrap();
  let resolved_str = resolved_from_cache.to_str().expect("path should be UTF-8");
  assert!(
    resolved_str.contains("/.yarn/berry/cache/path-to-regexp"),
    "Expected global cache path, got: {resolved_str}"
  );
}

#[tokio::test]
async fn resolve_pnp_transitive_dep_from_global_cache() {
  let fixture = super::fixture_root().join("pnp-global-cache-enabled");

  let resolver = Resolver::new(ResolveOptions {
    extensions: vec![".js".into()],
    enable_pnp: true,
    ..ResolveOptions::default()
  });

  let path_to_regexp = resolver
    .resolve(&fixture, "path-to-regexp")
    .await
    .map(|r| r.full_path())
    .unwrap();

  let module_root = path_to_regexp.parent().unwrap();

  let isarray = resolver
    .resolve(module_root, "isarray")
    .await
    .map(|r| {
      r.full_path()
        .to_str()
        .expect("path should be UTF-8")
        .cow_replace('\\', "/")
        .cow_to_lowercase()
        .into_owned()
    })
    .unwrap();

  assert!(
    isarray.contains(
      "yarn/berry/cache/isarray-npm-0.0.1-92e37e0a70-10c0.zip/node_modules/isarray/index.js"
    ),
    "Expected isarray in global cache, got: {isarray}"
  );
}
