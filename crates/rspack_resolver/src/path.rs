//! Path Utilities
//!
//! Code adapted from the following libraries
//! * [path-absolutize](https://docs.rs/path-absolutize)
//! * [normalize_path](https://docs.rs/normalize-path)
use camino::{Utf8Component, Utf8Path, Utf8PathBuf};

pub const SLASH_START: &[char; 2] = &['/', '\\'];

/// Extension trait to add path normalization to camino's [`Utf8Path`].
pub trait PathUtil {
  /// Normalize this path without performing I/O.
  ///
  /// All redundant separator and up-level references are collapsed.
  ///
  /// However, this does not resolve links.
  fn normalize(&self) -> Utf8PathBuf;

  /// Normalize with subpath assuming this path is normalized without performing I/O.
  ///
  /// All redundant separator and up-level references are collapsed.
  ///
  /// However, this does not resolve links.
  fn normalize_with<P: AsRef<Utf8Path>>(&self, subpath: P) -> Utf8PathBuf;

  /// Defined in ESM PACKAGE_TARGET_RESOLVE
  /// If target split on "/" or "\" contains any "", ".", "..", or "node_modules" segments after the first "." segment, case insensitive and including percent encoded variants
  fn is_invalid_exports_target(&self) -> bool;
}

impl PathUtil for Utf8Path {
  // https://github.com/parcel-bundler/parcel/blob/e0b99c2a42e9109a9ecbd6f537844a1b33e7faf5/packages/utils/node-resolver-rs/src/path.rs#L7
  fn normalize(&self) -> Utf8PathBuf {
    let mut components = self.components().peekable();
    let mut ret = if let Some(c @ Utf8Component::Prefix(..)) = components.peek() {
      let buf = Utf8PathBuf::from(c.as_str());
      components.next();
      buf
    } else {
      Utf8PathBuf::new()
    };

    for component in components {
      match component {
        Utf8Component::Prefix(..) => unreachable!("Path {:?}", self),
        Utf8Component::RootDir => {
          ret.push(component.as_str());
        }
        Utf8Component::CurDir => {}
        Utf8Component::ParentDir => {
          ret.pop();
        }
        Utf8Component::Normal(c) => {
          ret.push(c);
        }
      }
    }

    ret
  }

  // https://github.com/parcel-bundler/parcel/blob/e0b99c2a42e9109a9ecbd6f537844a1b33e7faf5/packages/utils/node-resolver-rs/src/path.rs#L37
  fn normalize_with<B: AsRef<Self>>(&self, subpath: B) -> Utf8PathBuf {
    let subpath = subpath.as_ref();

    let mut components = subpath.components();

    let Some(head) = components.next() else {
      return subpath.to_path_buf();
    };

    if matches!(head, Utf8Component::Prefix(..) | Utf8Component::RootDir) {
      return subpath.to_path_buf();
    }

    let mut ret = self.to_path_buf();
    for component in std::iter::once(head).chain(components) {
      match component {
        Utf8Component::CurDir => {}
        Utf8Component::ParentDir => {
          ret.pop();
        }
        Utf8Component::Normal(c) => {
          ret.push(c);
        }
        Utf8Component::Prefix(..) | Utf8Component::RootDir => {
          unreachable!("Path {:?} Subpath {:?}", self, subpath)
        }
      }
    }

    ret
  }

  fn is_invalid_exports_target(&self) -> bool {
    self.components().enumerate().any(|(index, c)| match c {
      Utf8Component::ParentDir => true,
      Utf8Component::CurDir => index > 0,
      Utf8Component::Normal(c) => c.eq_ignore_ascii_case("node_modules"),
      _ => false,
    })
  }
}

// https://github.com/webpack/enhanced-resolve/blob/main/test/path.test.js
#[tokio::test]
async fn is_invalid_exports_target() {
  let test_cases = [
    "../a.js",
    "../",
    "./a/b/../../../c.js",
    "./a/b/../../../",
    "./../../c.js",
    "./../../",
    "./a/../b/../../c.js",
    "./a/../b/../../",
    "./././../",
  ];

  for case in test_cases {
    assert!(Utf8Path::new(case).is_invalid_exports_target(), "{case}");
  }

  assert!(!Utf8Path::new("C:").is_invalid_exports_target());
  assert!(!Utf8Path::new("/").is_invalid_exports_target());
}

#[tokio::test]
async fn normalize() {
  assert_eq!(
    Utf8Path::new("/foo/.././foo/").normalize(),
    Utf8Path::new("/foo")
  );
  assert_eq!(Utf8Path::new("C://").normalize(), Utf8Path::new("C://"));
  assert_eq!(Utf8Path::new("C:").normalize(), Utf8Path::new("C:"));
  assert_eq!(
    Utf8Path::new(r"\\server\share").normalize(),
    Utf8Path::new(r"\\server\share")
  );
}
