//! <https://github.com/webpack/enhanced-resolve/blob/main/test/roots.test.js>

use std::path::PathBuf;

use crate::{AliasValue, ResolveError, ResolveOptions, Resolver};

fn dirname() -> PathBuf {
  super::fixture_root().join("enhanced_resolve").join("test")
}

#[tokio::test]
async fn roots() {
  let f = super::fixture();

  let resolver = Resolver::new(ResolveOptions {
    extensions: vec![".js".into()],
    alias: vec![("foo".into(), vec![AliasValue::from("/fixtures")])],
    roots: vec![dirname(), f.clone()],
    ..ResolveOptions::default()
  });

  #[rustfmt::skip]
    let pass = [
        ("should respect roots option", "/fixtures/b.js", f.join("b.js")),
        ("should try another root option, if it exists", "/b.js", f.join("b.js")),
        ("should respect extension", "/fixtures/b", f.join("b.js")),
        ("should resolve in directory", "/fixtures/extensions/dir", f.join("extensions/dir/index.js")),
        ("should respect aliases", "foo/b", f.join("b.js")),
    ];

  for (comment, request, expected) in pass {
    let resolved_path = resolver.resolve(&f, request).await.map(|r| r.full_path());
    assert_eq!(resolved_path, Ok(expected), "{comment} {request}");
  }

  #[rustfmt::skip]
    let fail = [
        ("should not work with relative path", "fixtures/b.js", ResolveError::NotFound("fixtures/b.js".into()))
    ];

  for (comment, request, expected) in fail {
    let resolution = resolver.resolve(&f, request).await;
    assert_eq!(resolution, Err(expected), "{comment} {request}");
  }
}

#[tokio::test]
async fn resolve_to_context() {
  let f = super::fixture();
  let resolver = Resolver::new(ResolveOptions {
    roots: vec![dirname(), f.clone()],
    resolve_to_context: true,
    ..ResolveOptions::default()
  });
  let resolved_path = resolver
    .resolve(&f, "/fixtures/lib")
    .await
    .map(|r| r.full_path());
  let expected = f.join("lib");
  assert_eq!(resolved_path, Ok(expected));
}

#[tokio::test]
async fn prefer_absolute() {
  let f = super::fixture();
  let resolver = Resolver::new(ResolveOptions {
    extensions: vec![".js".into()],
    alias: vec![("foo".into(), vec![AliasValue::from("/fixtures")])],
    roots: vec![dirname(), f.clone()],
    prefer_absolute: true,
    ..ResolveOptions::default()
  });

  #[rustfmt::skip]
    let pass = [
        ("should resolve an absolute path (prefer absolute)", f.join("b.js").to_str().expect("path should be UTF-8").to_string(), f.join("b.js")),
    ];

  for (comment, request, expected) in pass {
    let resolved_path = resolver.resolve(&f, &request).await.map(|r| r.full_path());
    assert_eq!(resolved_path, Ok(expected), "{comment} {request}");
  }
}

#[tokio::test]
async fn roots_fall_through() {
  let f = super::fixture();
  let absolute_path = f.join("roots_fall_through/index.js");
  let specifier = absolute_path.to_str().expect("path should be UTF-8");
  let resolution = Resolver::new(ResolveOptions::default().with_root(&f))
    .resolve(&f, &specifier)
    .await;
  assert_eq!(
    resolution.map(super::super::resolution::Resolution::into_path_buf),
    Ok(absolute_path)
  );
}
