//! <https://github.com/webpack/enhanced-resolve/blob/main/test/browserField.test.js>

use crate::{AliasValue, ResolveError, ResolveOptions, Resolver};

#[tokio::test]
async fn ignore() {
  let f = super::fixture().join("browser-module");

  let resolver = Resolver::new(ResolveOptions {
    alias_fields: vec![
      vec!["browser".into()],
      vec!["innerBrowser1".into()],
      vec!["innerBrowser2".into()],
      vec![],
    ],
    ..ResolveOptions::default()
  });

  #[rustfmt::skip]
    let data = [
        (f.clone(), "./lib/ignore", f.join("lib/ignore.js")),
        (f.clone(), "./lib/ignore.js", f.join("lib/ignore.js")),
        (f.join("lib"), "./ignore", f.join("lib/ignore.js")),
        (f.join("lib"), "./ignore.js", f.join("lib/ignore.js")),
    ];

  for (path, request, expected) in data {
    let resolution = resolver.resolve(&path, request).await;
    let expected = ResolveError::Ignored(expected);
    assert_eq!(resolution, Err(expected), "{path:?} {request}");
  }
}

#[tokio::test]
async fn shared_resolvers() {
  let f = super::fixture().join("browser-module");

  let resolver1 = Resolver::new(ResolveOptions {
    alias_fields: vec![vec![
      "innerBrowser1".into(),
      "field".into(),
      "browser".into(),
    ]],
    ..ResolveOptions::default()
  });
  let resolved_path = resolver1
    .resolve(&f, "./lib/main1.js")
    .await
    .map(|r| r.full_path());
  assert_eq!(resolved_path, Ok(f.join("lib/main.js")));

  let resolver2 = resolver1.clone_with_options(ResolveOptions {
    alias_fields: vec![vec!["innerBrowser2".into(), "browser".into()]],
    ..ResolveOptions::default()
  });
  let resolved_path = resolver2
    .resolve(&f, "./lib/main2.js")
    .await
    .map(|r| r.full_path());
  assert_eq!(resolved_path, Ok(f.join("./lib/replaced.js")));
}

#[tokio::test]
async fn replace_file() {
  let f = super::fixture().join("browser-module");

  let resolver = Resolver::new(ResolveOptions {
    alias_fields: vec![
      vec!["browser".into()],
      vec!["innerBrowser1".into(), "field2".into(), "browser".into()], // not presented
      vec!["innerBrowser1".into(), "field".into(), "browser".into()],
      vec!["innerBrowser2".into(), "browser".into()],
    ],
    // Not part of enhanced-resolve. Added to make sure no interaction between these two fields.
    main_fields: vec!["browser".into()],
    ..ResolveOptions::default()
  });

  #[rustfmt::skip]
    let data = [
        ("should replace a file 1", f.clone(), "./lib/replaced", f.join("lib/browser.js")),
        ("should replace a file 2", f.clone(), "./lib/replaced.js", f.join("lib/browser.js")),
        ("should replace a file 3", f.join("lib"), "./replaced", f.join("lib/browser.js")),
        ("should replace a file 4", f.join("lib"), "./replaced.js", f.join("lib/browser.js")),
        ("should replace a module with a file 1", f.clone(), "module-a", f.join("browser/module-a.js")),
        ("should replace a module with a file 2", f.join("lib"), "module-a", f.join("browser/module-a.js")),
        ("should replace a module with a module 1", f.clone(), "module-b", f.join("node_modules/module-c.js")),
        ("should replace a module with a module 2", f.join("lib"), "module-b", f.join("node_modules/module-c.js")),
        ("should resolve in nested property 1", f.clone(), "./lib/main1.js", f.join("lib/main.js")),
        ("should resolve in nested property 2", f.clone(), "./lib/main2.js", f.join("lib/browser.js")),
        ("should check only alias field properties", f.clone(), "./toString", f.join("lib/toString.js")),
        // not part of enhanced-resolve
        ("recursion", f.clone(), "module-c", f.join("node_modules/module-c.js")),
        ("resolve self 1", f.clone(), "./lib/main.js", f.join("lib/main.js")),
        ("resolve self 2", f.clone(), "./main.js", f.join("lib/main.js")),
    ];

  for (comment, path, request, expected) in data {
    let resolved_path = resolver
      .resolve(&path, request)
      .await
      .map(|r| r.full_path());
    assert_eq!(resolved_path, Ok(expected), "{comment} {path:?} {request}");
  }
}

#[tokio::test]
async fn recurse_fail() {
  let f = super::fixture();

  let resolver = Resolver::new(ResolveOptions {
    alias_fields: vec![vec!["browser".into()]],
    ..ResolveOptions::default()
  });

  #[rustfmt::skip]
    let data = [
        ("recurse non existent", f.clone(), "./lib/non-existent.js", ResolveError::NotFound("./lib/non-existent.js".into())),
        ("path partial match 1", f.clone(), "./xyz.js", ResolveError::NotFound("./xyz.js".into())),
        ("path partial match 2", f, "./lib/xyz.js", ResolveError::NotFound("./lib/xyz.js".into())),
    ];

  for (comment, path, request, expected) in data {
    let resolved_path = resolver
      .resolve(&path, request)
      .await
      .map(|r| r.full_path());
    assert_eq!(resolved_path, Err(expected), "{comment} {path:?} {request}");
  }
}

#[tokio::test]
async fn broken() {
  let f = super::fixture();

  let resolver = Resolver::new(ResolveOptions {
    alias_fields: vec![vec!["browser".into()]],
    ..ResolveOptions::default()
  });

  #[rustfmt::skip]
    let data = [
        // The browser field string value should be ignored
        (f.clone(), "browser-module-broken", Ok(f.join("node_modules/browser-module-broken/main.js"))),
        (f.join("browser-module"), "./number", Err(ResolveError::NotFound("./number".into()))),
    ];

  for (path, request, expected) in data {
    let resolved_path = resolver
      .resolve(&path, request)
      .await
      .map(|r| r.full_path());
    assert_eq!(resolved_path, expected, "{path:?} {request}");
  }
}

#[tokio::test]
async fn crypto_js() {
  let f = super::fixture();

  let resolver = Resolver::new(ResolveOptions {
    alias_fields: vec![vec!["browser".into()]],
    fallback: vec![(
      "crypto".into(),
      vec![AliasValue::from(
        f.join("lib.js").to_str().expect("path should be UTF-8"),
      )],
    )],
    ..ResolveOptions::default()
  });

  let resolved_path = resolver
    .resolve(f.join("crypto-js"), "crypto")
    .await
    .map(|r| r.full_path());
  assert_eq!(
    resolved_path,
    Err(ResolveError::Ignored(f.join("crypto-js")))
  );
}

// https://github.com/webpack/webpack/blob/87660921808566ef3b8796f8df61bd79fc026108/test/cases/resolving/browser-field/index.js#L40-L43
#[tokio::test]
async fn recursive() {
  let f = super::fixture().join("browser-module");

  let resolver = Resolver::new(ResolveOptions {
    alias_fields: vec![vec!["browser".into()]],
    ..ResolveOptions::default()
  });

  let data = [
    (
      "should handle recursive file 1",
      f.clone(),
      "recursive-file/a",
    ),
    (
      "should handle recursive file 2",
      f.clone(),
      "recursive-file/b",
    ),
    (
      "should handle recursive file 3",
      f.clone(),
      "recursive-file/c",
    ),
    ("should handle recursive file 4", f, "recursive-file/d"),
  ];

  for (comment, path, request) in data {
    let resolved_path = resolver.resolve(&path, request).await;
    assert_eq!(
      resolved_path,
      Err(ResolveError::Recursion),
      "{comment} {path:?} {request}"
    );
  }
}

#[tokio::test]
async fn with_query() {
  let f = super::fixture().join("browser-module");

  let resolver = Resolver::new(ResolveOptions {
    alias_fields: vec![vec!["browser".into()]],
    ..ResolveOptions::default()
  });

  let resolved_path = resolver.resolve(&f, "./foo").await.map(|r| r.full_path());
  assert_eq!(resolved_path, Ok(f.join("lib").join("browser.js?query")));
}
