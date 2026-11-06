//! Path Utilities
//!
//! Windows path classification and normalization follow enhanced-resolve and Node.js
//! independently of the host platform. POSIX paths retain the resolver's existing
//! traversal and separator conventions.
use camino::{Utf8Component, Utf8Path, Utf8PathBuf};

pub const SLASH_START: &[char; 2] = &['/', '\\'];

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PathType {
  Empty,
  Normal,
  Relative,
  AbsoluteWindows,
  AbsolutePosix,
  Internal,
}

impl PathType {
  pub(crate) fn of(path: &str) -> Self {
    let bytes = path.as_bytes();
    match bytes.first() {
      None => Self::Empty,
      Some(b'/') => Self::AbsolutePosix,
      Some(b'#') => Self::Internal,
      Some(b'.')
        if matches!(bytes, b"." | b"..") || path.starts_with("./") || path.starts_with("../") =>
      {
        Self::Relative
      }
      _ if bytes.len() >= 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes.len() == 2 || matches!(bytes[2], b'/' | b'\\')) =>
      {
        Self::AbsoluteWindows
      }
      _ if path.starts_with(r"\\") => Self::AbsoluteWindows,
      _ => Self::Normal,
    }
  }

  pub(crate) fn is_absolute(&self) -> bool {
    matches!(self, Self::AbsoluteWindows | Self::AbsolutePosix)
  }
}

fn normalize_segments(mut result: String, tail: &str, windows: bool, rooted: bool) -> Utf8PathBuf {
  let separator = if windows { '\\' } else { '/' };
  let root_len = result.len();
  for segment in tail.split(|c| c == '/' || (windows && c == '\\')) {
    match segment {
      "" | "." => {}
      ".."
        if result.len() > root_len && result[root_len..].rsplit(separator).next() != Some("..") =>
      {
        let end = result[root_len..]
          .rfind(separator)
          .map_or(root_len, |i| root_len + i);
        result.truncate(end);
      }
      ".." if rooted || !windows => {}
      _ => {
        if result.len() > root_len {
          result.push(separator);
        }
        result.push_str(segment);
      }
    }
  }
  if result.len() == root_len && !rooted && windows {
    result.push('.');
  }
  if windows && result.len() > root_len && tail.ends_with(['/', '\\']) {
    result.push(separator);
  }
  result.into()
}

fn normalize_posix(path: &str) -> Utf8PathBuf {
  let rooted = path.starts_with('/');
  let mut result = String::with_capacity(path.len() + 1);
  if rooted {
    result.push('/');
  }
  normalize_segments(result, path, false, rooted)
}

fn is_windows_reserved_device(name: &str) -> bool {
  ["CON", "PRN", "AUX", "NUL"]
    .iter()
    .any(|reserved| name.eq_ignore_ascii_case(reserved))
    || (name.get(..3).is_some_and(|prefix| {
      prefix.eq_ignore_ascii_case("COM") || prefix.eq_ignore_ascii_case("LPT")
    }) && matches!(
      name.get(3..),
      Some("1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³")
    ))
}

fn normalize_windows(path: &str) -> Utf8PathBuf {
  let bytes = path.as_bytes();
  let is_separator = |byte: u8| matches!(byte, b'/' | b'\\');
  let mut result = String::with_capacity(path.len() + 1);
  let mut root_end = 0;
  let mut rooted = false;
  if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
    result.push_str(&path[..2]);
    root_end = 2;
    rooted = bytes.get(2).is_some_and(|&b| is_separator(b));
    if rooted {
      result.push('\\');
      root_end = 3;
    }
  } else if bytes.first().is_some_and(|&b| is_separator(b)) {
    rooted = true;
    root_end = 1;
    result.push('\\');
    if bytes.get(1).is_some_and(|&b| is_separator(b)) {
      let server_end = bytes[2..]
        .iter()
        .position(|&b| is_separator(b))
        .map(|i| i + 2);
      if let Some(server_end) = server_end.filter(|&end| end > 2) {
        let share_start = bytes[server_end..]
          .iter()
          .position(|&b| !is_separator(b))
          .map(|i| i + server_end);
        if let Some(share_start) = share_start {
          let share_end = bytes[share_start..]
            .iter()
            .position(|&b| is_separator(b))
            .map_or(bytes.len(), |i| i + share_start);
          result.push('\\');
          let server = &path[2..server_end];
          result.push_str(server);
          result.push('\\');
          if matches!(server, "." | "?") {
            root_end = share_start;
            if let Some(colon) = path[share_start..]
              .find(':')
              .map(|i| i + share_start)
              .filter(|&colon| is_windows_reserved_device(&path[share_start..colon]))
            {
              result.clear();
              result.push_str(r"\\?\");
              result.push_str(&path[share_start..=colon]);
              result.push('\\');
              root_end = colon + 1;
            }
          } else {
            result.push_str(&path[share_start..share_end]);
            result.push('\\');
            root_end = share_end;
          }
        }
      }
    }
  }
  normalize_segments(result, &path[root_end..], true, rooted)
}

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
  fn normalize(&self) -> Utf8PathBuf {
    match PathType::of(self.as_str()) {
      PathType::Empty => Utf8PathBuf::new(),
      PathType::AbsoluteWindows => normalize_windows(self.as_str()),
      _ => normalize_posix(self.as_str()),
    }
  }

  fn normalize_with<B: AsRef<Self>>(&self, subpath: B) -> Utf8PathBuf {
    let subpath = subpath.as_ref();
    let subpath_type = PathType::of(subpath.as_str());
    if subpath_type == PathType::Empty {
      return self.normalize();
    }
    if subpath_type.is_absolute() {
      return subpath.normalize();
    }
    match PathType::of(self.as_str()) {
      PathType::AbsoluteWindows => normalize_windows(&format!(r"{self}\{subpath}")),
      PathType::Empty => subpath.normalize(),
      _ => normalize_posix(&format!("{self}/{subpath}")),
    }
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
