//! <https://github.com/webpack/enhanced-resolve/blob/main/test/restrictions.test.js>

use std::sync::Arc;

use regex::Regex;

use crate::{ResolveError, ResolveOptions, Resolver, Restriction};

#[tokio::test]
async fn should_respect_regexp_restriction() {
  let f = super::fixture().join("restrictions");

  let re = Regex::new(r"\.(sass|scss|css)$").unwrap();
  let resolver1 = Resolver::new(ResolveOptions {
    extensions: vec![".js".into()],
    restrictions: vec![Restriction::Fn(Arc::new(move |path| {
      path.as_os_str().to_str().map_or(false, |s| re.is_match(s))
    }))],
    ..ResolveOptions::default()
  });

  let resolution = resolver1.resolve(&f, "pck1").await.map(|r| r.full_path());
  assert_eq!(resolution, Err(ResolveError::NotFound("pck1".to_string())));
}

#[tokio::test]
async fn should_try_to_find_alternative_1() {
  let f = super::fixture().join("restrictions");

  let re = Regex::new(r"\.(sass|scss|css)$").unwrap();
  let resolver1 = Resolver::new(ResolveOptions {
    extensions: vec![".js".into(), ".css".into()],
    main_files: vec!["index".into()],
    restrictions: vec![Restriction::Fn(Arc::new(move |path| {
      path.as_os_str().to_str().map_or(false, |s| re.is_match(s))
    }))],
    ..ResolveOptions::default()
  });

  let resolution = resolver1.resolve(&f, "pck1").await.map(|r| r.full_path());
  assert_eq!(resolution, Ok(f.join("node_modules/pck1/index.css")));
}

#[tokio::test]
async fn should_respect_string_restriction() {
  let fixture = super::fixture();
  let f = fixture.join("restrictions");

  let resolver = Resolver::new(ResolveOptions {
    extensions: vec![".js".into()],
    restrictions: vec![Restriction::Path(f.clone())],
    ..ResolveOptions::default()
  });

  let resolution = resolver.resolve(&f, "pck2").await;
  assert_eq!(resolution, Err(ResolveError::NotFound("pck2".to_string())));
}

#[tokio::test]
async fn should_allow_descendant_of_string_restriction() {
  let f = super::fixture().join("restrictions");

  let resolver = Resolver::new(ResolveOptions {
    extensions: vec![".js".into()],
    restrictions: vec![Restriction::Path(f.clone())],
    ..ResolveOptions::default()
  });

  let resolution = resolver.resolve(&f, "pck1").await.map(|r| r.full_path());
  assert_eq!(resolution, Ok(f.join("node_modules/pck1/index.js")));
}

#[tokio::test]
async fn should_reject_sibling_sharing_textual_prefix() {
  let f = super::fixture().join("restrictions");

  let resolver = Resolver::new(ResolveOptions {
    extensions: vec![".js".into()],
    restrictions: vec![Restriction::Path(f.join("node_modules/pck"))],
    ..ResolveOptions::default()
  });

  let resolution = resolver.resolve(&f, "pck1").await.map(|r| r.full_path());
  assert_eq!(resolution, Err(ResolveError::NotFound("pck1".to_string())));
}

#[tokio::test]
async fn should_try_to_find_alternative_2() {
  let f = super::fixture().join("restrictions");

  let re = Regex::new(r"\.(sass|scss|css)$").unwrap();
  let resolver1 = Resolver::new(ResolveOptions {
    extensions: vec![".js".into(), ".css".into()],
    main_fields: vec!["main".into(), "style".into()],
    restrictions: vec![Restriction::Fn(Arc::new(move |path| {
      path.as_os_str().to_str().map_or(false, |s| re.is_match(s))
    }))],
    ..ResolveOptions::default()
  });

  let resolution = resolver1.resolve(&f, "pck2").await.map(|r| r.full_path());
  assert_eq!(resolution, Ok(f.join("node_modules/pck2/index.css")));
}

#[tokio::test]
async fn should_try_to_find_alternative_3() {
  let f = super::fixture().join("restrictions");

  let re = Regex::new(r"\.(sass|scss|css)$").unwrap();
  let resolver1 = Resolver::new(ResolveOptions {
    extensions: vec![".js".into()],
    main_fields: vec!["main".into(), "module".into(), "style".into()],
    restrictions: vec![Restriction::Fn(Arc::new(move |path| {
      path.as_os_str().to_str().map_or(false, |s| re.is_match(s))
    }))],
    ..ResolveOptions::default()
  });

  let resolution = resolver1.resolve(&f, "pck2").await.map(|r| r.full_path());
  assert_eq!(resolution, Ok(f.join("node_modules/pck2/index.css")));
}

#[tokio::test]
async fn should_try_to_find_alternative_4() {
  let f = super::fixture().join("restrictions");

  let re = Regex::new(r"\.(sass|scss|css)$").unwrap();
  let resolver1 = Resolver::new(ResolveOptions {
    extensions: vec![".css".into()],
    main_fields: vec!["main".into()],
    extension_alias: vec![(".js".into(), vec![".js".into(), ".jsx".into()])],
    restrictions: vec![Restriction::Fn(Arc::new(move |path| {
      path.as_os_str().to_str().map_or(false, |s| re.is_match(s))
    }))],
    ..ResolveOptions::default()
  });

  let resolution = resolver1.resolve(&f, "pck2").await.map(|r| r.full_path());
  assert_eq!(resolution, Ok(f.join("node_modules/pck2/index.css")));
}

/// Ported from enhanced-resolve `restrictions > path boundaries`
/// <https://github.com/webpack/enhanced-resolve/commit/d8693b6>
///
/// `MemoryFS` always separates with `/`, so these run on non-Windows only.
#[cfg(not(target_os = "windows"))]
mod path_boundaries {
  use std::path::PathBuf;

  use super::super::memory_fs::MemoryFS;
  use crate::{ResolveOptions, ResolverGeneric, Restriction};

  async fn resolves(restriction: &str, context: &str, request: &str, file: &'static str) -> bool {
    let resolver = ResolverGeneric::<MemoryFS>::new_with_file_system(
      MemoryFS::new(&[(file, "")]),
      ResolveOptions {
        extensions: vec![".js".into()],
        restrictions: vec![Restriction::Path(PathBuf::from(restriction))],
        ..ResolveOptions::default()
      },
    );
    resolver.resolve(context, request).await.is_ok()
  }

  #[tokio::test]
  async fn file_inside_a_restriction() {
    assert!(resolves("/a/b/c", "/a/b/c", "./index.js", "/a/b/c/index.js").await);
  }

  #[tokio::test]
  async fn sibling_of_a_restriction() {
    assert!(!resolves("/a/b/c", "/a/b", "./c-other.js", "/a/b/c-other.js").await);
  }

  #[tokio::test]
  async fn sibling_of_a_restriction_separated_by_a_backslash() {
    assert!(!resolves("/a/b/c", "/a/b", "./c\\sibling.js", "/a/b/c\\sibling.js").await);
  }

  #[tokio::test]
  async fn sibling_of_a_restriction_containing_a_backslash() {
    assert!(!resolves("/a/b\\c", "/a", "./b\\c\\sibling.js", "/a/b\\c\\sibling.js").await);
  }

  #[tokio::test]
  async fn file_inside_a_restriction_ending_with_a_separator() {
    assert!(resolves("/a/b/c/", "/a/b/c", "./index.js", "/a/b/c/index.js").await);
  }

  #[tokio::test]
  async fn file_inside_the_root_restriction() {
    assert!(resolves("/", "/a", "./index.js", "/a/index.js").await);
  }

  #[tokio::test]
  async fn file_inside_a_non_normalized_restriction() {
    assert!(resolves("/a/x/../b/c", "/a/b/c", "./index.js", "/a/b/c/index.js").await);
  }

  #[tokio::test]
  async fn empty_restriction_matches_every_path() {
    assert!(resolves("", "/a/b/c", "./index.js", "/a/b/c/index.js").await);
  }

  #[tokio::test]
  async fn relative_restriction_matches_no_absolute_path() {
    assert!(!resolves(".", "/a/b/c", "./index.js", "/a/b/c/index.js").await);
    assert!(!resolves("..", "/a/b/c", "./index.js", "/a/b/c/index.js").await);
    assert!(!resolves("foo/..", "/a/b/c", "./index.js", "/a/b/c/index.js").await);
  }

  #[tokio::test]
  async fn file_inside_a_restriction_differing_in_case() {
    assert!(!resolves("/A/B/C", "/a/b/c", "./index.js", "/a/b/c/index.js").await);
  }
}

/// Ported from enhanced-resolve `restrictions > windows and posix semantics`
/// <https://github.com/webpack/enhanced-resolve/commit/d8693b6>
///
/// Windows path forms cannot travel through a resolver on a posix host, so the
/// check is driven directly, the way upstream drives its plugin.
#[cfg(target_os = "windows")]
mod windows_path_boundaries {
  use std::path::PathBuf;

  use camino::Utf8Path;

  use crate::{ResolveOptions, Resolver, Restriction};

  fn is_inside(restriction: &str, path: &str) -> bool {
    let resolver = Resolver::new(ResolveOptions {
      restrictions: vec![Restriction::Path(PathBuf::from(restriction))],
      ..ResolveOptions::default()
    });
    resolver.check_restrictions(Utf8Path::new(path))
  }

  #[tokio::test]
  async fn path_using_slashes_under_a_backslash_restriction() {
    assert!(is_inside(r"C:\a\b\c", "C:/a/b/c/index.js"));
  }

  #[tokio::test]
  async fn path_mixing_separators_under_a_slash_restriction() {
    assert!(is_inside("C:/a/b/c", r"C:\a\b/c/index.js"));
  }

  #[tokio::test]
  async fn path_under_a_restriction_ending_with_a_slash() {
    assert!(is_inside(r"C:\a\b\c/", r"C:\a\b\c\index.js"));
  }

  #[tokio::test]
  async fn the_restricted_directory_itself() {
    assert!(is_inside(r"C:\a\b\c\", r"C:\a\b\c"));
  }

  #[tokio::test]
  async fn path_whose_drive_letter_differs_in_case() {
    assert!(is_inside(r"c:\a\b\c", r"C:\a\b\c\index.js"));
  }

  #[tokio::test]
  async fn path_inside_a_unc_restriction() {
    assert!(is_inside(r"\\server\share\a", r"\\server\share\a\index.js"));
  }

  #[tokio::test]
  async fn path_inside_a_dos_device_restriction() {
    assert!(is_inside(r"\\?\C:\a", r"\\?\C:\a\index.js"));
  }

  #[tokio::test]
  async fn path_on_another_share_than_the_unc_restriction() {
    assert!(!is_inside(
      r"\\server\share\a",
      r"\\server\other\a\index.js"
    ));
  }

  #[tokio::test]
  async fn sibling_of_a_restriction_written_with_slashes() {
    assert!(!is_inside(r"C:\a\b\c", "C:/a/b/c-other.js"));
  }
}

/// Ported from enhanced-resolve `restrictions > with symlinks`
/// <https://github.com/webpack/enhanced-resolve/pull/595> (GHSA-fvr2-82rg-p3pp)
///
/// A restriction has to hold for the path the resolver returns, not for the
/// spelling a candidate happened to have while it was being selected.
mod escaping_the_restriction {
  use std::{
    fs, io,
    path::{Path, PathBuf},
  };

  use crate::{ResolveError, ResolveOptions, Resolver, Restriction};

  fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> io::Result<()> {
    #[cfg(target_family = "unix")]
    return std::os::unix::fs::symlink(original, link);
    #[cfg(target_family = "windows")]
    return std::os::windows::fs::symlink_file(original, link);
  }

  /// `allowed/{link,rel-link}.js` point at `outside/secret.js`.
  /// `None` when the platform refuses to create symlinks.
  fn fixture(name: &str) -> Option<PathBuf> {
    let root = std::env::temp_dir().join(name);
    _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("allowed")).ok()?;
    fs::create_dir_all(root.join("outside")).ok()?;
    // `/var` is a symlink on macOS, so the restriction has to name the real
    // directory. Windows answers with a `\\?\` path, where `join` collapses the
    // `..` these tests are about and the resolver cannot take it as a specifier.
    let canonical = root.canonicalize().ok()?;
    let canonical = canonical.to_str()?;
    let root = PathBuf::from(canonical.strip_prefix(r"\\?\").unwrap_or(canonical));
    fs::write(root.join("outside/secret.js"), "").ok()?;
    fs::write(root.join("allowed/real.js"), "").ok()?;
    symlink(root.join("outside/secret.js"), root.join("allowed/link.js")).ok()?;
    symlink(
      Path::new("../outside/secret.js"),
      root.join("allowed/rel-link.js"),
    )
    .ok()?;
    Some(root)
  }

  async fn resolve(root: &Path, specifier: &str) -> Result<PathBuf, ResolveError> {
    resolve_with(root, specifier, true).await
  }

  async fn resolve_with(
    root: &Path,
    specifier: &str,
    symlinks: bool,
  ) -> Result<PathBuf, ResolveError> {
    let allowed = root.join("allowed");
    Resolver::new(ResolveOptions {
      extensions: vec![".js".into()],
      restrictions: vec![Restriction::Path(allowed.clone())],
      symlinks,
      ..ResolveOptions::default()
    })
    .resolve(&allowed, specifier)
    .await
    .map(|r| r.full_path())
  }

  #[tokio::test]
  async fn in_root_symlink_to_an_outside_target() {
    let Some(root) = fixture("rspack_resolver_restriction_symlink") else {
      return;
    };
    assert_eq!(
      resolve(&root, "./link.js").await,
      Err(ResolveError::NotFound("./link.js".to_string()))
    );
  }

  #[tokio::test]
  async fn in_root_relative_symlink_to_an_outside_target() {
    let Some(root) = fixture("rspack_resolver_restriction_rel_symlink") else {
      return;
    };
    assert_eq!(
      resolve(&root, "./rel-link.js").await,
      Err(ResolveError::NotFound("./rel-link.js".to_string()))
    );
  }

  #[tokio::test]
  async fn parent_dir_traversal_in_an_absolute_specifier() {
    let Some(root) = fixture("rspack_resolver_restriction_traversal") else {
      return;
    };
    let escaping = root.join("allowed/../outside/secret.js");
    let escaping = escaping.to_str().unwrap();
    assert_eq!(
      resolve(&root, escaping).await,
      Err(ResolveError::NotFound(escaping.to_string()))
    );
  }

  #[tokio::test]
  async fn parent_dir_traversal_without_symlink_resolution() {
    let Some(root) = fixture("rspack_resolver_restriction_traversal_nosym") else {
      return;
    };
    let escaping = root.join("allowed/../outside/secret.js");
    let escaping = escaping.to_str().unwrap();
    assert_eq!(
      resolve_with(&root, escaping, false).await,
      Err(ResolveError::NotFound(escaping.to_string()))
    );
  }

  #[tokio::test]
  async fn real_in_root_file_still_resolves() {
    let Some(root) = fixture("rspack_resolver_restriction_real") else {
      return;
    };
    assert_eq!(
      resolve(&root, "./real.js").await,
      Ok(root.join("allowed/real.js"))
    );
  }
}
