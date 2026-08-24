//! https://github.com/webpack/enhanced-resolve/blob/main/test/dependencies.test.js

// Warm-cache calls must report the same missing_dependencies as the cold path so that
// webpack/rspack watchers can track non-existent node_modules dirs across multiple resolves.
#[cfg(not(target_os = "windows"))]
mod warm_cache_missing_dependencies {
  use std::path::PathBuf;

  use super::super::memory_fs::MemoryFS;
  use crate::{ResolveContext, ResolveOptions, ResolverGeneric};

  fn file_system() -> MemoryFS {
    MemoryFS::new(&[
      ("/a/b/c/some.js", ""), // makes /a/b/c and /a/b real dirs; neither has node_modules
      ("/a/node_modules/module/index.js", ""),
    ])
  }

  #[tokio::test]
  async fn node_modules_missing_deps_same_on_warm_cache() {
    let resolver = ResolverGeneric::<MemoryFS>::new_with_file_system(
      file_system(),
      ResolveOptions {
        extensions: vec![".js".into()],
        ..ResolveOptions::default()
      },
    );

    let path = PathBuf::from("/a/b/c");

    let mut ctx_cold = ResolveContext::default();
    let cold = resolver
      .resolve_with_context(path.clone(), "module", &mut ctx_cold)
      .await;

    let mut ctx_warm = ResolveContext::default();
    let warm = resolver
      .resolve_with_context(path.clone(), "module", &mut ctx_warm)
      .await;

    assert_eq!(cold.map(|r| r.full_path()), warm.map(|r| r.full_path()));

    assert_eq!(
      ctx_cold.missing_dependencies, ctx_warm.missing_dependencies,
      "cold: {:?}\nwarm: {:?}",
      ctx_cold.missing_dependencies, ctx_warm.missing_dependencies,
    );

    // enhanced-resolve lists traversed-but-absent node_modules dirs in missingDependencies
    for dir in ["/a/b/c/node_modules", "/a/b/node_modules"] {
      assert!(
        ctx_warm
          .missing_dependencies
          .contains(&PathBuf::from(dir).into())
      );
    }
  }
}

#[cfg(not(target_os = "windows"))] // MemoryFS's path separator is always `/` so the test will not pass in windows.
mod windows {
  use std::path::PathBuf;

  use super::super::memory_fs::MemoryFS;
  use crate::{InternedPath, InternedPathSet, ResolveContext, ResolveOptions, ResolverGeneric};

  fn file_system() -> MemoryFS {
    MemoryFS::new(&[
      ("/a/b/node_modules/some-module/index.js", ""),
      (
        "/a/node_modules/module/package.json",
        r#"{"main":"entry.js"}"#,
      ),
      ("/a/node_modules/module/file.js", r#"{"main":"entry.js"}"#),
      ("/modules/other-module/file.js", ""),
    ])
  }

  #[tokio::test]
  async fn test() {
    let file_system = file_system();

    let resolver = ResolverGeneric::<MemoryFS>::new_with_file_system(
      file_system,
      ResolveOptions {
        extensions: vec![".json".into(), ".js".into()],
        modules: vec!["/modules".into(), "node_modules".into()],
        ..ResolveOptions::default()
      },
    );

    let data = [
      (
        "middle module request",
        "/a/b/c",
        "module/file",
        "/a/node_modules/module/file.js",
        // These dependencies are different from enhanced-resolve due to different code path to
        // querying the file system
        vec![
          // found package.json
          "/a/node_modules/module/package.json",
          // symlink checks
          "/a/node_modules/module/file.js",
          // "/a/node_modules/module",
          // "/a/node_modules",
          // "/a",
          // "/",
        ],
        vec![
          // missing package.jsons
          // "/a/b/c/package.json",
          "/a/b/package.json",
          "/a/package.json",
          "/package.json",
          // missing modules directories
          "/a/b/c",
          // "/a/b/c/node_modules",
          // missing single file modules
          "/modules/module",
          "/a/b/node_modules/module",
          // missing files with alternative extensions
          "/a/node_modules/module/file",
          "/a/node_modules/module/file.json",
        ],
      ),
      (
        "fast found module",
        "/a/b/c",
        "other-module/file.js",
        "/modules/other-module/file.js",
        // These dependencies are different from enhanced-resolve due to different code path to
        // querying the file system
        vec![
          // symlink checks
          "/modules/other-module/file.js",
          // "/modules/other-module",
          // "/modules",
          // "/",
        ],
        vec![
          // missing package.jsons
          // "/a/b/c/package.json",
          "/a/b/c",
          "/a/b/package.json",
          "/a/package.json",
          "/package.json",
          "/modules/other-module/package.json",
          "/modules/package.json",
        ],
      ),
    ];

    for (name, context, request, result, file_dependencies, missing_dependencies) in data {
      let mut ctx = ResolveContext::default();
      let path = PathBuf::from(context);
      let resolved = resolver
        .resolve_with_context(path, request, &mut ctx)
        .await
        .map(|r| r.full_path());
      assert_eq!(resolved, Ok(PathBuf::from(result)));
      let file_dependencies: InternedPathSet = file_dependencies
        .iter()
        .map(|p| InternedPath::from(PathBuf::from(p)))
        .collect();
      let missing_dependencies: InternedPathSet = missing_dependencies
        .iter()
        .map(|p| InternedPath::from(PathBuf::from(p)))
        .collect();
      assert_eq!(
        InternedPathSet::from_iter(ctx.file_dependencies.iter().cloned()),
        file_dependencies,
        "{name}"
      );
      assert_eq!(
        InternedPathSet::from_iter(ctx.missing_dependencies.iter().cloned()),
        missing_dependencies,
        "{name}"
      );
    }
  }
}
