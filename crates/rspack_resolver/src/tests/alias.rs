//! <https://github.com/webpack/enhanced-resolve/blob/main/test/alias.test.js>

use std::path::Path;

use normalize_path::NormalizePath;

use crate::{AliasValue, Resolution, ResolveContext, ResolveError, ResolveOptions, Resolver};

#[tokio::test]
#[cfg(not(target_os = "windows"))] // MemoryFS's path separator is always `/` so the test will not pass in windows.
async fn alias() {
  use std::path::{Path, PathBuf};

  use super::memory_fs::MemoryFS;
  use crate::ResolverGeneric;

  let f = Path::new("/");

  let file_system = MemoryFS::new(&[
    ("/a/index", ""),
    ("/a/dir/index", ""),
    ("/recursive/index", ""),
    ("/recursive/dir/index", ""),
    ("/b/index", ""),
    ("/b/dir/index", ""),
    ("/c/index", ""),
    ("/c/dir/index", ""),
    ("/d/index.js", ""),
    ("/d/dir/.empty", ""),
    ("/e/index", ""),
    ("/e/anotherDir/index", ""),
    ("/e/dir/file", ""),
    ("/dashed-name", ""),
  ]);

  let resolver = ResolverGeneric::<MemoryFS>::new_with_file_system(
    file_system,
    ResolveOptions {
      alias: vec![
        ("aliasA".into(), vec![AliasValue::from("a")]),
        ("b$".into(), vec![AliasValue::from("a/index")]),
        ("c$".into(), vec![AliasValue::from("/a/index")]),
        (
          "multiAlias".into(),
          vec![
            AliasValue::from("b"),
            AliasValue::from("c"),
            AliasValue::from("d"),
            AliasValue::from("e"),
            AliasValue::from("a"),
          ],
        ),
        ("recursive".into(), vec![AliasValue::from("recursive/dir")]),
        ("/d/dir".into(), vec![AliasValue::from("/c/dir")]),
        ("/d/index.js".into(), vec![AliasValue::from("/c/index")]),
        ("#".into(), vec![AliasValue::from("/c/dir")]),
        ("@".into(), vec![AliasValue::from("/c/dir")]),
        ("ignored".into(), vec![AliasValue::Ignore]),
        // not part of enhanced-resolve, added to make sure query in alias value works
        (
          "alias_query".into(),
          vec![AliasValue::from("a?query_after")],
        ),
        (
          "alias_fragment".into(),
          vec![AliasValue::from("a#fragment_after")],
        ),
        ("dash".into(), vec![AliasValue::Ignore]),
        (
          "@scope/package-name/file$".into(),
          vec![AliasValue::from("/c/dir")],
        ),
      ],
      modules: vec!["/".into()],
      ..ResolveOptions::default()
    },
  );

  #[rustfmt::skip]
    let pass = [
        ("should resolve a not aliased module 1", "a", "/a/index"),
        ("should resolve a not aliased module 2", "a/index", "/a/index"),
        ("should resolve a not aliased module 3", "a/dir", "/a/dir/index"),
        ("should resolve a not aliased module 4", "a/dir/index", "/a/dir/index"),
        ("should resolve an aliased module 1", "aliasA", "/a/index"),
        ("should resolve an aliased module 2", "aliasA/index", "/a/index"),
        ("should resolve an aliased module 3", "aliasA/dir", "/a/dir/index"),
        ("should resolve an aliased module 4", "aliasA/dir/index", "/a/dir/index"),
        ("should resolve '#' alias 1", "#", "/c/dir/index"),
        ("should resolve '#' alias 2", "#/index", "/c/dir/index"),
        ("should resolve '@' alias 1", "@", "/c/dir/index"),
        ("should resolve '@' alias 2", "@/index", "/c/dir/index"),
        ("should resolve '@' alias 3", "@/", "/c/dir/index"),
        ("should resolve a recursive aliased module 1", "recursive", "/recursive/dir/index"),
        ("should resolve a recursive aliased module 2", "recursive/index", "/recursive/dir/index"),
        ("should resolve a recursive aliased module 3", "recursive/dir", "/recursive/dir/index"),
        ("should resolve a recursive aliased module 4", "recursive/dir/index", "/recursive/dir/index"),
        ("should resolve a file aliased module 1", "b", "/a/index"),
        ("should resolve a file aliased module 2", "c", "/a/index"),
        ("should resolve a file aliased module with a query 1", "b?query", "/a/index?query"),
        ("should resolve a file aliased module with a query 2", "c?query", "/a/index?query"),
        ("should resolve a path in a file aliased module 1", "b/index", "/b/index"),
        ("should resolve a path in a file aliased module 2", "b/dir", "/b/dir/index"),
        ("should resolve a path in a file aliased module 3", "b/dir/index", "/b/dir/index"),
        ("should resolve a path in a file aliased module 4", "c/index", "/c/index"),
        ("should resolve a path in a file aliased module 5", "c/dir", "/c/dir/index"),
        ("should resolve a path in a file aliased module 6", "c/dir/index", "/c/dir/index"),
        ("should resolve a file aliased file 1", "d", "/c/index"),
        ("should resolve a file aliased file 2", "d/dir/index", "/c/dir/index"),
        ("should resolve a file in multiple aliased dirs 1", "multiAlias/dir/file", "/e/dir/file"),
        ("should resolve a file in multiple aliased dirs 2", "multiAlias/anotherDir", "/e/anotherDir/index"),
        // not part of enhanced-resolve, added to make sure query in alias value works
        ("should resolve query in alias value", "alias_query?query_before", "/a/index?query_after"),
        ("should resolve query in alias value", "alias_fragment#fragment_before", "/a/index#fragment_after"),
        ("should resolve dashed name", "dashed-name", "/dashed-name"),
        ("should resolve scoped package name with sub dir", "@scope/package-name/file", "/c/dir/index"),
    ];

  for (comment, request, expected) in pass {
    let resolved_path = resolver.resolve(f, request).await.map(|r| r.full_path());
    assert_eq!(
      resolved_path,
      Ok(PathBuf::from(expected)),
      "{comment} {request}"
    );
  }

  #[rustfmt::skip]
    let ignore = [
        ("should resolve an ignore module", "ignored", ResolveError::Ignored(f.join("ignored")))
    ];

  for (comment, request, expected) in ignore {
    let resolution = resolver.resolve(f, request).await;
    assert_eq!(resolution, Err(expected), "{comment} {request}");
  }
}

// Not part of enhanced-resolve
#[tokio::test]
async fn infinite_recursion() {
  let f = super::fixture();
  let resolver = Resolver::new(ResolveOptions {
    alias: vec![
      ("./a".into(), vec![AliasValue::from("./b")]),
      ("./b".into(), vec![AliasValue::from("./a")]),
    ],
    ..ResolveOptions::default()
  });
  let resolution = resolver.resolve(f, "./a").await;
  assert_eq!(resolution, Err(ResolveError::Recursion));
}

fn check_slash(path: &Path) {
  let s = path.to_str().expect("path should be UTF-8").to_string();
  #[cfg(target_os = "windows")]
  {
    assert!(!s.contains('/'), "{s}");
    assert!(s.contains('\\'), "{s}");
  }
  #[cfg(not(target_os = "windows"))]
  {
    assert!(s.contains('/'), "{s}");
    assert!(!s.contains('\\'), "{s}");
  }
}

#[tokio::test]
async fn absolute_path() {
  let f = super::fixture();
  let resolver = Resolver::new(ResolveOptions {
    alias: vec![(
      f.join("foo").to_str().unwrap().to_string(),
      vec![AliasValue::Ignore],
    )],
    modules: vec![f.clone().to_str().unwrap().to_string()],
    ..ResolveOptions::default()
  });
  let resolution = resolver.resolve(&f, "foo/index").await;
  assert_eq!(resolution, Err(ResolveError::Ignored(f.join("foo"))));
}

#[tokio::test]
async fn system_path() {
  let f = super::fixture();
  let resolver = Resolver::new(ResolveOptions {
    alias: vec![(
      "@app".into(),
      vec![AliasValue::from(
        f.join("alias").to_str().expect("path should be UTF-8"),
      )],
    )],
    ..ResolveOptions::default()
  });

  let specifiers = ["@app/files/a", "@app/files/a.js"];

  for specifier in specifiers {
    let path = resolver
      .resolve(&f, specifier)
      .await
      .map(Resolution::into_path_buf)
      .unwrap();
    assert_eq!(path, f.join("alias/files/a.js"));
    check_slash(&path);
  }
}

#[tokio::test]
async fn alias_is_full_path() {
  let f = super::fixture();
  let dir = f.join("foo");
  let dir_str = dir.to_str().expect("path should be UTF-8").to_string();

  let resolver = Resolver::new(ResolveOptions {
    alias: vec![("@".into(), vec![AliasValue::Path(dir_str.clone())])],
    ..ResolveOptions::default()
  });

  let mut ctx = ResolveContext::default();

  let specifiers = [
    "@/index".to_string(),
    "@/index.js".to_string(),
    // specifier has multiple `/` for reasons we'll never know
    "@////index".to_string(),
    // specifier is a full path
    dir_str,
  ];

  for specifier in specifiers {
    let resolution = resolver.resolve_with_context(&f, &specifier, &mut ctx);
    assert_eq!(
      resolution.await.map(|r| r.full_path()),
      Ok(dir.join("index.js"))
    );
  }

  for path in ctx.file_dependencies {
    assert_eq!(path.as_path(), path.normalize(), "{path:?}");
    check_slash(&path);
  }

  for path in ctx.missing_dependencies {
    assert_eq!(path.as_path(), path.normalize(), "{path:?}");
    check_slash(&path);
    if let Some(path) = path.parent() {
      assert!(!path.is_file(), "{path:?} must not be a file");
    }
  }
}

// For the `should_stop` variable in `load_alias`
#[tokio::test]
async fn all_alias_values_are_not_found() {
  let f = super::fixture();
  let resolver = Resolver::new(ResolveOptions {
    alias: vec![(
      "m1".to_string(),
      vec![AliasValue::Path(
        f.join("node_modules")
          .join("m2")
          .to_str()
          .expect("path should be UTF-8")
          .to_string(),
      )],
    )],
    ..ResolveOptions::default()
  });
  let resolution = resolver.resolve(&f, "m1/a.js").await;
  assert_eq!(
    resolution,
    Err(ResolveError::MatchedAliasNotFound(
      "m1/a.js".to_string(),
      "m1".to_string(),
    ))
  );
}

#[tokio::test]
async fn alias_fragment() {
  let f = super::fixture();

  let data = [
    // enhanced-resolve has `#` prepended with a `\0`, they are removed from the
    // following 3 expected test results.
    // See https://github.com/webpack/enhanced-resolve#escaping
    (
      "handle fragment edge case (no fragment)",
      "./no#fragment/#/#",
      f.join("no#fragment/#/#.js"),
    ),
    (
      "handle fragment edge case (fragment)",
      "./no#fragment/#/",
      f.join("no.js#fragment/#/"),
    ),
    (
      "handle fragment escaping",
      "./no\0#fragment/\0#/\0##fragment",
      f.join("no#fragment/#/#.js#fragment"),
    ),
  ];

  for (comment, request, expected) in data {
    let resolver = Resolver::new(ResolveOptions {
      alias: vec![(
        "foo".to_string(),
        vec![AliasValue::Path(request.to_string())],
      )],
      ..ResolveOptions::default()
    });
    let resolved_path = resolver.resolve(&f, "foo").await.map(|r| r.full_path());
    assert_eq!(resolved_path, Ok(expected), "{comment} {request}");
  }
}

#[tokio::test]
async fn alias_try_fragment_as_path() {
  let f = super::fixture();
  let resolver = Resolver::new(ResolveOptions {
    alias: vec![(
      "#".to_string(),
      vec![AliasValue::Path(
        f.join("#")
          .to_str()
          .expect("path should be UTF-8")
          .to_string(),
      )],
    )],
    ..ResolveOptions::default()
  });
  let resolution = resolver.resolve(&f, "#/a").await.map(|r| r.full_path());
  assert_eq!(resolution, Ok(f.join("#").join("a.js")));
}
