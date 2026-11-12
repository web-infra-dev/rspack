//! <https://github.com/webpack/enhanced-resolve/blob/main/test/simple.test.js>

use std::env;

use crate::Resolver;

#[tokio::test]
async fn resolve_abs_main() {
  let resolver = Resolver::default();
  let dirname = env::current_dir().unwrap().join("fixtures");
  let f = dirname.join("invalid/main.js");
  // a's main field id `/dist/index.js`
  let resolution = resolver.resolve(&f, "a").await.unwrap();

  assert_eq!(
    resolution.path(),
    dirname.join("invalid/node_modules/a/dist/index.js")
  );
}

#[tokio::test]
async fn simple() {
  // mimic `enhanced-resolve/test/simple.test.js`
  let dirname = env::current_dir().unwrap().join("fixtures");
  let f = dirname.join("enhanced_resolve/test");

  let resolver = Resolver::default();

  let data = [
    ("direct", f.clone(), "../lib/index"),
    ("as directory", f, ".."),
    ("as module", dirname.clone(), "./enhanced_resolve"),
  ];

  for (comment, path, request) in data {
    let resolved_path = resolver
      .resolve(&path, request)
      .await
      .map(|f| f.full_path());
    let expected = dirname.join("enhanced_resolve/lib/index.js");
    assert_eq!(resolved_path, Ok(expected), "{comment} {path:?} {request}");
  }
}

#[tokio::test]
async fn dashed_name() {
  let f = super::fixture();

  let resolver = Resolver::default();

  let data = [
    (f.clone(), "dash", f.join("node_modules/dash/index.js")),
    (
      f.clone(),
      "dash-name",
      f.join("node_modules/dash-name/index.js"),
    ),
    (
      f.join("node_modules/dash"),
      "dash",
      f.join("node_modules/dash/index.js"),
    ),
    (
      f.join("node_modules/dash"),
      "dash-name",
      f.join("node_modules/dash-name/index.js"),
    ),
    (
      f.join("node_modules/dash-name"),
      "dash",
      f.join("node_modules/dash/index.js"),
    ),
    (
      f.join("node_modules/dash-name"),
      "dash-name",
      f.join("node_modules/dash-name/index.js"),
    ),
  ];

  for (path, request, expected) in data {
    let resolved_path = resolver
      .resolve(&path, request)
      .await
      .map(|f| f.full_path());
    assert_eq!(resolved_path, Ok(expected), "{path:?} {request}");
  }
}

#[cfg(not(target_os = "windows"))] // MemoryFS's path separator is always `/` so the test will not pass in windows.
mod windows {
  use super::super::memory_fs::MemoryFS;
  use crate::ResolveOptions;

  #[tokio::test]
  async fn no_package() {
    use std::path::Path;

    use crate::ResolverGeneric;
    let f = Path::new("/");
    let file_system = MemoryFS::new(&[]);
    let resolver =
      ResolverGeneric::<MemoryFS>::new_with_file_system(file_system, ResolveOptions::default());
    let resolved_path = resolver.resolve(f, "package").await;
    assert!(resolved_path.is_err());
  }
}
