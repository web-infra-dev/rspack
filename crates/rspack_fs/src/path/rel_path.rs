use std::{
  fmt,
  path::{Path, PathBuf},
  str::FromStr,
};

use rspack_cacheable::cacheable;

use super::normalize::{is_canonical_rel_path, normalize_rel_join, normalize_rel_path};
use crate::{Error, Result};

/// A lexically normalized, strictly UTF-8, portable POSIX (`/`-separated) relative path.
///
/// The empty string `""` denotes the current directory (`.`).
#[cacheable]
#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelPath(String);

impl RelPath {
  pub fn new(raw: &str) -> Result<Self> {
    Ok(Self(normalize_rel_path(raw)?.into_owned()))
  }

  pub fn new_contained(raw: &str) -> Result<Self> {
    let rel = Self::new(raw)?;
    if rel.escapes_root() {
      return Err(Error::PathEscapesRoot(raw.to_string()));
    }
    Ok(rel)
  }

  #[inline]
  pub(crate) fn from_normalized(normalized: String) -> Self {
    debug_assert!(is_canonical_rel_path(&normalized));
    Self(normalized)
  }

  #[inline]
  pub fn try_from_utf8_path(path: &camino::Utf8Path) -> Result<Self> {
    Self::new(path.as_str())
  }

  pub fn try_from_std_path(path: &Path) -> Result<Self> {
    let raw = path
      .to_str()
      .ok_or_else(|| Error::NonUtf8Path(path.to_path_buf()))?;
    Self::new(raw)
  }

  #[inline]
  pub fn as_str(&self) -> &str {
    &self.0
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  #[inline]
  pub fn escapes_root(&self) -> bool {
    self.0 == ".." || self.0.starts_with("../")
  }

  #[inline]
  pub fn as_utf8_path(&self) -> &camino::Utf8Path {
    camino::Utf8Path::new(&self.0)
  }

  #[inline]
  pub fn to_utf8_path_buf(&self) -> camino::Utf8PathBuf {
    camino::Utf8PathBuf::from(&self.0)
  }

  #[inline]
  pub fn as_std_path(&self) -> &Path {
    Path::new(&self.0)
  }

  #[inline]
  pub fn to_std_path_buf(&self) -> PathBuf {
    PathBuf::from(&self.0)
  }

  #[inline]
  pub fn file_name(&self) -> Option<&str> {
    if self.0.is_empty() || self.0 == ".." || self.0.ends_with("/..") {
      return None;
    }
    self.0.rsplit('/').next()
  }

  #[inline]
  pub fn file_stem(&self) -> Option<&str> {
    self
      .file_name()
      .and_then(|name| camino::Utf8Path::new(name).file_stem())
  }

  #[inline]
  pub fn extension(&self) -> Option<&str> {
    self
      .file_name()
      .and_then(|name| camino::Utf8Path::new(name).extension())
  }

  pub fn parent(&self) -> Option<Self> {
    if self.0.is_empty() || self.0 == ".." || self.0.ends_with("/..") {
      return None;
    }
    match self.0.rsplit_once('/') {
      Some((parent, _)) => Some(Self(parent.to_string())),
      None => Some(Self(String::new())),
    }
  }

  pub fn join(&self, sub: &str) -> Result<Self> {
    Ok(Self(normalize_rel_join(&self.0, sub)?))
  }

  pub fn join_rel(&self, sub: &Self) -> Self {
    if sub.is_empty() {
      return self.clone();
    }
    if self.is_empty() {
      return sub.clone();
    }
    Self::from_normalized(
      normalize_rel_join(&self.0, &sub.0)
        .expect("validated RelPath cannot fail normalize_rel_join"),
    )
  }

  #[inline]
  pub fn strip_prefix(&self, base: &Self) -> Option<&str> {
    if base.0.is_empty() {
      return Some(&self.0);
    }
    if self == base {
      return Some("");
    }
    self.0.strip_prefix(&base.0)?.strip_prefix('/')
  }

  #[inline]
  pub fn starts_with(&self, base: &Self) -> bool {
    self.strip_prefix(base).is_some()
  }
}

impl fmt::Debug for RelPath {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl fmt::Display for RelPath {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.0)
  }
}

impl AsRef<str> for RelPath {
  #[inline]
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl AsRef<Path> for RelPath {
  #[inline]
  fn as_ref(&self) -> &Path {
    self.as_std_path()
  }
}

impl AsRef<camino::Utf8Path> for RelPath {
  #[inline]
  fn as_ref(&self) -> &camino::Utf8Path {
    self.as_utf8_path()
  }
}

impl FromStr for RelPath {
  type Err = Error;

  #[inline]
  fn from_str(value: &str) -> Result<Self> {
    Self::new(value)
  }
}

impl TryFrom<&str> for RelPath {
  type Error = Error;

  #[inline]
  fn try_from(value: &str) -> Result<Self> {
    Self::new(value)
  }
}

impl TryFrom<&camino::Utf8Path> for RelPath {
  type Error = Error;

  #[inline]
  fn try_from(value: &camino::Utf8Path) -> Result<Self> {
    Self::try_from_utf8_path(value)
  }
}

impl TryFrom<&camino::Utf8PathBuf> for RelPath {
  type Error = Error;

  #[inline]
  fn try_from(value: &camino::Utf8PathBuf) -> Result<Self> {
    Self::try_from_utf8_path(value.as_path())
  }
}

impl TryFrom<camino::Utf8PathBuf> for RelPath {
  type Error = Error;

  #[inline]
  fn try_from(value: camino::Utf8PathBuf) -> Result<Self> {
    Self::try_from_utf8_path(value.as_path())
  }
}

impl TryFrom<&Path> for RelPath {
  type Error = Error;

  #[inline]
  fn try_from(value: &Path) -> Result<Self> {
    Self::try_from_std_path(value)
  }
}

impl TryFrom<&PathBuf> for RelPath {
  type Error = Error;

  #[inline]
  fn try_from(value: &PathBuf) -> Result<Self> {
    Self::try_from_std_path(value.as_path())
  }
}

impl TryFrom<PathBuf> for RelPath {
  type Error = Error;

  #[inline]
  fn try_from(value: PathBuf) -> Result<Self> {
    Self::try_from_std_path(value.as_path())
  }
}

impl From<RelPath> for String {
  #[inline]
  fn from(value: RelPath) -> Self {
    value.0
  }
}

impl From<RelPath> for camino::Utf8PathBuf {
  #[inline]
  fn from(value: RelPath) -> Self {
    value.to_utf8_path_buf()
  }
}

impl From<&RelPath> for PathBuf {
  #[inline]
  fn from(value: &RelPath) -> Self {
    value.to_std_path_buf()
  }
}

impl From<RelPath> for PathBuf {
  #[inline]
  fn from(value: RelPath) -> Self {
    value.to_std_path_buf()
  }
}
