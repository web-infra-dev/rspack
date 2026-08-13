//! <https://github.com/webpack/enhanced-resolve/blob/main/test/extensions.test.js>

use rustc_hash::FxHashSet;

use crate::{
  EnforceExtension, Resolution, ResolveContext, ResolveError, ResolveOptions, Resolver,
  ResolverPath,
};

#[tokio::test]
async fn extensions() {
  let f = super::fixture().join("extensions");

  let resolver = Resolver::new(ResolveOptions {
    extensions: vec![".ts".into(), ".js".into()],
    ..ResolveOptions::default()
  });

  #[rustfmt::skip]
    let pass = [
        ("should resolve according to order of provided extensions", "./foo", "foo.ts"),
        ("should resolve according to order of provided extensions (dir index)", "./dir", "dir/index.ts"),
        ("should resolve according to main field in module root", ".", "index.js"),
        // This is a core module
        // ("should resolve single file module before directory", "module", "node_modules/module.js"),
        ("should resolve trailing slash directory before single file", "module/", "node_modules/module/index.ts"),
    ];

  for (comment, request, expected_path) in pass {
    let resolved_path = resolver.resolve(&f, request).await.map(|r| r.full_path());
    let expected = f.join(expected_path);
    assert_eq!(
      resolved_path,
      Ok(expected),
      "{comment} {request} {expected_path}"
    );
  }

  #[rustfmt::skip]
    let fail = [
        ("not resolve to file when request has a trailing slash (relative)", "./foo.js/", "./foo.js/".into())
    ];

  for (comment, request, expected_error) in fail {
    let resolution = resolver.resolve(&f, request).await;
    let error = ResolveError::NotFound(expected_error);
    assert_eq!(resolution, Err(error), "{comment} {request} {resolution:?}");
  }
}

// should default enforceExtension to true when extensions includes an empty string
#[tokio::test]
async fn default_enforce_extension() {
  let f = super::fixture().join("extensions");

  let mut ctx = ResolveContext::default();
  let resolved = Resolver::new(ResolveOptions {
    extensions: vec![".ts".into(), String::new(), ".js".into()],
    ..ResolveOptions::default()
  })
  .resolve_with_context(&f, "./foo", &mut ctx)
  .await;

  assert_eq!(
    resolved.map(Resolution::into_path_buf),
    Ok(f.join("foo.ts"))
  );
  assert_eq!(
    ctx.file_dependencies,
    FxHashSet::from_iter([
      ResolverPath::from(f.join("foo.ts")),
      ResolverPath::from(f.join("package.json")),
    ])
  );
  assert!(ctx.missing_dependencies.is_empty());
}

// should respect enforceExtension when extensions includes an empty string
#[tokio::test]
async fn respect_enforce_extension() {
  let f = super::fixture().join("extensions");

  let mut ctx = ResolveContext::default();
  let resolved = Resolver::new(ResolveOptions {
    enforce_extension: EnforceExtension::Disabled,
    extensions: vec![".ts".into(), String::new(), ".js".into()],
    ..ResolveOptions::default()
  })
  .resolve_with_context(&f, "./foo", &mut ctx)
  .await;

  assert_eq!(
    resolved.map(Resolution::into_path_buf),
    Ok(f.join("foo.ts"))
  );
  assert_eq!(
    ctx.file_dependencies,
    FxHashSet::from_iter([
      ResolverPath::from(f.join("foo.ts")),
      ResolverPath::from(f.join("package.json")),
    ])
  );
  assert_eq!(
    ctx.missing_dependencies,
    FxHashSet::from_iter([ResolverPath::from(f.join("foo"))])
  );
}

#[tokio::test]
async fn multi_dot_extension() {
  let f = super::fixture().join("extensions");

  let resolver = Resolver::new(ResolveOptions {
    // Test for `.d.ts`, not part of enhanced-resolve.
    extensions: vec![".a.b.c".into(), ".d.ts".into(), ".ts".into(), ".js".into()],
    ..ResolveOptions::default()
  });

  #[rustfmt::skip]
    let pass = [
        ("should resolve according to order of provided extensions", "./foo", "foo.ts"),
        ("should resolve file with extension", "./app.module", "app.module.js")
    ];

  for (comment, request, expected_path) in pass {
    let resolved_path = resolver.resolve(&f, request).await.map(|r| r.full_path());
    let expected = f.join(expected_path);
    assert_eq!(
      resolved_path,
      Ok(expected),
      "{comment} {request} {expected_path}"
    );
  }

  #[rustfmt::skip]
    let fail = [
        ("not resolve to file", "./index.", "./index.".into())
    ];

  for (comment, request, expected_error) in fail {
    let resolution = resolver.resolve(&f, request).await;
    let error = ResolveError::NotFound(expected_error);
    assert_eq!(resolution, Err(error), "{comment} {request} {resolution:?}");
  }
}

#[test]
#[should_panic = "All extensions must start with a leading dot"]
fn without_leading_dot() {
  Resolver::new(ResolveOptions {
    extensions: vec!["ts".into(), "js".into()],
    ..ResolveOptions::default()
  });
}
