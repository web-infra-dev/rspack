use std::{
  fmt,
  path::{Path, PathBuf},
  str::FromStr,
};

use super::{abs_path::AbsPath, rel_path::RelPath};
use crate::{Error, Result};

/// A normalized absolute or relative path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AbsOrRelPath {
  Abs(AbsPath),
  Rel(RelPath),
}

impl AbsOrRelPath {
  pub fn new(raw: &str) -> Result<Self> {
    match AbsPath::new(raw) {
      Ok(abs) => Ok(Self::Abs(abs)),
      Err(Error::NotAbsolutePath(_)) => RelPath::new(raw).map(Self::Rel),
      Err(err) => Err(err),
    }
  }

  pub fn try_from_std_path(path: &Path) -> Result<Self> {
    let raw = path
      .to_str()
      .ok_or_else(|| Error::NonUtf8Path(path.to_path_buf()))?;
    Self::new(raw)
  }

  #[inline]
  pub fn try_from_utf8_path(path: &camino::Utf8Path) -> Result<Self> {
    Self::new(path.as_str())
  }

  #[inline]
  pub const fn is_absolute(&self) -> bool {
    matches!(self, Self::Abs(_))
  }

  #[inline]
  pub const fn is_relative(&self) -> bool {
    matches!(self, Self::Rel(_))
  }

  #[inline]
  pub fn as_str(&self) -> &str {
    match self {
      Self::Abs(abs) => abs.as_str(),
      Self::Rel(rel) => rel.as_str(),
    }
  }

  #[inline]
  pub fn as_abs(&self) -> Option<&AbsPath> {
    match self {
      Self::Abs(abs) => Some(abs),
      Self::Rel(_) => None,
    }
  }

  #[inline]
  pub fn as_rel(&self) -> Option<&RelPath> {
    match self {
      Self::Abs(_) => None,
      Self::Rel(rel) => Some(rel),
    }
  }

  #[inline]
  pub fn resolve(&self, base: &AbsPath) -> AbsPath {
    match self {
      Self::Abs(abs) => abs.clone(),
      Self::Rel(rel) => base.join_rel(rel),
    }
  }

  #[inline]
  pub fn as_std_path(&self) -> &Path {
    Path::new(self.as_str())
  }

  #[inline]
  pub fn to_std_path_buf(&self) -> PathBuf {
    PathBuf::from(self.as_str())
  }
}

impl fmt::Display for AbsOrRelPath {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}

impl AsRef<str> for AbsOrRelPath {
  #[inline]
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl From<AbsPath> for AbsOrRelPath {
  #[inline]
  fn from(value: AbsPath) -> Self {
    Self::Abs(value)
  }
}

impl From<RelPath> for AbsOrRelPath {
  #[inline]
  fn from(value: RelPath) -> Self {
    Self::Rel(value)
  }
}

impl FromStr for AbsOrRelPath {
  type Err = Error;

  #[inline]
  fn from_str(value: &str) -> Result<Self> {
    Self::new(value)
  }
}

impl TryFrom<&str> for AbsOrRelPath {
  type Error = Error;

  #[inline]
  fn try_from(value: &str) -> Result<Self> {
    Self::new(value)
  }
}
