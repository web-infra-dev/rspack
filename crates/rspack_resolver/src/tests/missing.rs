//! https://github.com/webpack/enhanced-resolve/blob/main/test/missing.test.js

use normalize_path::NormalizePath;

use crate::{AliasValue, ResolveContext, ResolveOptions, Resolver, ResolverPath};

#[tokio::test]
async fn test() {
  let f = super::fixture();

  let data = [
    (
      "./missing-file",
      vec![
        f.join("missing-file"),
        f.join("missing-file.js"),
        f.join("missing-file.node"),
      ],
    ),
    (
      "missing-module",
      vec![
        f.join("node_modules/missing-module"),
        f.parent().unwrap().join("node_modules"), // enhanced-resolve is "node_modules/missing-module"
      ],
    ),
    (
      "missing-module/missing-file",
      vec![
        f.join("node_modules/missing-module"),
        // f.parent().unwrap().join("node_modules/missing-module"), // we don't report this
      ],
    ),
    (
      "m1/missing-file",
      vec![
        f.join("node_modules/m1/missing-file"),
        f.join("node_modules/m1/missing-file.js"),
        f.join("node_modules/m1/missing-file.node"),
        // f.parent().unwrap().join("node_modules/m1"), // we don't report this
      ],
    ),
    (
      "m1/",
      vec![
        f.join("node_modules/m1/index"),
        f.join("node_modules/m1/index.js"),
        f.join("node_modules/m1/index.json"),
        f.join("node_modules/m1/index.node"),
      ],
    ),
    ("m1/a", vec![f.join("node_modules/m1/a")]),
  ];

  let resolver = Resolver::default();

  for (specifier, missing_dependencies) in data {
    let mut ctx = ResolveContext::default();
    let _ = resolver.resolve_with_context(&f, specifier, &mut ctx).await;

    for path in ctx.file_dependencies {
      assert_eq!(path.as_path(), path.normalize(), "{path:?}");
    }

    for path in missing_dependencies {
      assert_eq!(path, path.normalize(), "{path:?}");
      assert!(
        ctx
          .missing_dependencies
          .contains(&ResolverPath::from(&path)),
        "{specifier}: {path:?} not in {:?}",
        &ctx.missing_dependencies
      );
    }
  }
}

#[tokio::test]
async fn alias_and_extensions() {
  let f = super::fixture();

  let resolver = Resolver::new(ResolveOptions {
    alias: vec![
      (
        "@scope-js/package-name/dir$".into(),
        vec![AliasValue::Path(
          f.join("foo/index.js")
            .to_str()
            .expect("path should be UTF-8")
            .to_string(),
        )],
      ),
      (
        "react-dom".into(),
        vec![AliasValue::Path(
          f.join("foo/index.js")
            .to_str()
            .expect("path should be UTF-8")
            .to_string(),
        )],
      ),
    ],
    extensions: vec![".server.ts".into()],

    ..ResolveOptions::default()
  });

  let mut ctx = ResolveContext::default();
  let _ = resolver.resolve_with_context(&f, "@scope-js/package-name/dir/router", &mut ctx);
  let _ = resolver.resolve_with_context(&f, "react-dom/client", &mut ctx);

  for path in ctx.file_dependencies {
    assert_eq!(path.as_path(), path.normalize(), "{path:?}");
  }

  for path in ctx.missing_dependencies {
    assert_eq!(path.as_path(), path.normalize(), "{path:?}");
    if let Some(path) = path.parent() {
      assert!(!path.is_file(), "{path:?} must not be a file");
    }
  }
}
