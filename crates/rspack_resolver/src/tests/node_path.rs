use std::path::{Path, PathBuf};

use super::memory_fs::MemoryFS;
use crate::{ResolveOptions, ResolverGeneric};

fn new_resolver(
  node_path_dirs: Vec<PathBuf>,
  data: &[(&'static str, &'static str)],
) -> ResolverGeneric<MemoryFS> {
  ResolverGeneric::<MemoryFS>::new_with_file_system(
    MemoryFS::new(data),
    ResolveOptions {
      node_path: true,
      ..ResolveOptions::default()
    },
  )
  .with_node_path_dirs(node_path_dirs)
}

#[cfg(not(target_os = "windows"))]
mod posix {
  use super::*;

  #[tokio::test]
  async fn basic() {
    let resolver = new_resolver(
      vec![PathBuf::from("/global/lib")],
      &[
        ("/global/lib/foo/package.json", r#"{"main":"index.js"}"#),
        ("/global/lib/foo/index.js", ""),
      ],
    );
    let result = resolver.resolve(Path::new("/project/src"), "foo").await;
    assert_eq!(
      result.map(|r| r.full_path()),
      Ok(PathBuf::from("/global/lib/foo/index.js"))
    );
  }

  #[tokio::test]
  async fn node_modules_takes_priority_over_node_path() {
    let resolver = new_resolver(
      vec![PathBuf::from("/global/lib")],
      &[
        (
          "/project/node_modules/foo/package.json",
          r#"{"main":"index.js"}"#,
        ),
        ("/project/node_modules/foo/index.js", ""),
        ("/global/lib/foo/package.json", r#"{"main":"index.js"}"#),
        ("/global/lib/foo/index.js", ""),
      ],
    );
    let result = resolver.resolve(Path::new("/project/src"), "foo").await;
    assert_eq!(
      result.map(|r| r.full_path()),
      Ok(PathBuf::from("/project/node_modules/foo/index.js")),
    );
  }

  #[tokio::test]
  async fn multiple_node_path_dirs() {
    let resolver = new_resolver(
      vec![PathBuf::from("/first/lib"), PathBuf::from("/second/lib")],
      &[
        ("/first/lib/foo/package.json", r#"{"main":"index.js"}"#),
        ("/first/lib/foo/index.js", ""),
        ("/second/lib/bar/package.json", r#"{"main":"index.js"}"#),
        ("/second/lib/bar/index.js", ""),
      ],
    );
    let result = resolver.resolve(Path::new("/project"), "foo").await;
    assert_eq!(
      result.map(|r| r.full_path()),
      Ok(PathBuf::from("/first/lib/foo/index.js"))
    );

    let result = resolver.resolve(Path::new("/project"), "bar").await;
    assert_eq!(
      result.map(|r| r.full_path()),
      Ok(PathBuf::from("/second/lib/bar/index.js"))
    );
  }

  #[tokio::test]
  async fn first_node_path_dir_takes_priority() {
    let resolver = new_resolver(
      vec![PathBuf::from("/first/lib"), PathBuf::from("/second/lib")],
      &[
        ("/first/lib/foo/package.json", r#"{"main":"a.js"}"#),
        ("/first/lib/foo/a.js", ""),
        ("/second/lib/foo/package.json", r#"{"main":"b.js"}"#),
        ("/second/lib/foo/b.js", ""),
      ],
    );
    let result = resolver.resolve(Path::new("/project"), "foo").await;
    assert_eq!(
      result.map(|r| r.full_path()),
      Ok(PathBuf::from("/first/lib/foo/a.js"))
    );
  }

  #[tokio::test]
  async fn nonexistent_node_path_dir_skipped() {
    let resolver = new_resolver(
      vec![PathBuf::from("/nonexistent"), PathBuf::from("/global/lib")],
      &[
        ("/global/lib/foo/package.json", r#"{"main":"index.js"}"#),
        ("/global/lib/foo/index.js", ""),
      ],
    );
    let result = resolver.resolve(Path::new("/project"), "foo").await;
    assert_eq!(
      result.map(|r| r.full_path()),
      Ok(PathBuf::from("/global/lib/foo/index.js"))
    );
  }

  #[tokio::test]
  async fn scoped_package() {
    let resolver = new_resolver(
      vec![PathBuf::from("/global/lib")],
      &[
        (
          "/global/lib/@scope/pkg/package.json",
          r#"{"main":"index.js"}"#,
        ),
        ("/global/lib/@scope/pkg/index.js", ""),
      ],
    );
    let result = resolver.resolve(Path::new("/project"), "@scope/pkg").await;
    assert_eq!(
      result.map(|r| r.full_path()),
      Ok(PathBuf::from("/global/lib/@scope/pkg/index.js"))
    );
  }

  #[tokio::test]
  async fn subpath_import() {
    let resolver = new_resolver(
      vec![PathBuf::from("/global/lib")],
      &[
        ("/global/lib/foo/package.json", r#"{"main":"index.js"}"#),
        ("/global/lib/foo/index.js", ""),
        ("/global/lib/foo/lib/utils.js", ""),
      ],
    );
    let result = resolver
      .resolve(Path::new("/project"), "foo/lib/utils")
      .await;
    assert_eq!(
      result.map(|r| r.full_path()),
      Ok(PathBuf::from("/global/lib/foo/lib/utils.js"))
    );
  }

  #[tokio::test]
  async fn disabled_node_path() {
    let resolver = ResolverGeneric::<MemoryFS>::new_with_file_system(
      MemoryFS::new(&[
        ("/global/lib/foo/package.json", r#"{"main":"index.js"}"#),
        ("/global/lib/foo/index.js", ""),
      ]),
      ResolveOptions {
        node_path: false,
        ..ResolveOptions::default()
      },
    );
    let result = resolver.resolve(Path::new("/project"), "foo").await;
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn not_found_in_node_path() {
    let resolver = new_resolver(
      vec![PathBuf::from("/global/lib")],
      &[
        ("/global/lib/bar/package.json", r#"{"main":"index.js"}"#),
        ("/global/lib/bar/index.js", ""),
      ],
    );
    let result = resolver.resolve(Path::new("/project"), "foo").await;
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn file_without_package_json() {
    let resolver = new_resolver(
      vec![PathBuf::from("/global/lib")],
      &[("/global/lib/foo.js", "")],
    );
    let result = resolver.resolve(Path::new("/project"), "foo").await;
    assert_eq!(
      result.map(|r| r.full_path()),
      Ok(PathBuf::from("/global/lib/foo.js"))
    );
  }

  #[tokio::test]
  async fn resolve_with_extensions() {
    let resolver = ResolverGeneric::<MemoryFS>::new_with_file_system(
      MemoryFS::new(&[("/global/lib/foo.json", "{}")]),
      ResolveOptions {
        node_path: true,
        extensions: vec![".js".into(), ".json".into()],
        ..ResolveOptions::default()
      },
    )
    .with_node_path_dirs(vec![PathBuf::from("/global/lib")]);
    let result = resolver.resolve(Path::new("/project"), "foo").await;
    assert_eq!(
      result.map(|r| r.full_path()),
      Ok(PathBuf::from("/global/lib/foo.json"))
    );
  }

  #[tokio::test]
  async fn index_file_resolution() {
    let resolver = new_resolver(
      vec![PathBuf::from("/global/lib")],
      &[("/global/lib/foo/index.js", "")],
    );
    let result = resolver.resolve(Path::new("/project"), "foo").await;
    assert_eq!(
      result.map(|r| r.full_path()),
      Ok(PathBuf::from("/global/lib/foo/index.js"))
    );
  }
}
