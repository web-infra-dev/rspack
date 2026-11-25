use std::{
  cmp::Ordering,
  fmt,
  hash::{Hash, Hasher},
  path::{Path, PathBuf},
};

use rspack_intern::{InternSliceStorage, InternedSlice, SliceInternable};
use rustc_hash::FxHasher;

use super::normalize::{PathFlavor, abs_join, abs_parent, abs_strip_prefix, normalize_abs_path};
use crate::{Error, Result};

struct AbsPathInner;

impl SliceInternable for AbsPathInner {
  type Header = u64;
  type Item = u8;

  #[inline]
  fn hash(header: &u64, _bytes: &[u8]) -> u64 {
    *header
  }

  #[inline]
  fn eq(a: &[u8], b: &[u8]) -> bool {
    a == b
  }

  fn storage() -> &'static InternSliceStorage<Self> {
    static STORAGE: InternSliceStorage<AbsPathInner> = InternSliceStorage::new();
    &STORAGE
  }
}

#[inline]
pub(crate) fn fx_hash_bytes(bytes: &[u8]) -> u64 {
  let mut hasher = FxHasher::default();
  hasher.write(bytes);
  hasher.finish()
}

/// An interned, strictly UTF-8, lexically normalized absolute OS path.
///
/// Intentionally does NOT implement `rspack_cacheable` / `rkyv` traits so unanchored
/// absolute paths cannot be persisted directly into `new_cache`.
#[derive(Clone, PartialEq, Eq)]
pub struct AbsPath(InternedSlice<AbsPathInner>);

impl AbsPath {
  pub fn new(raw: &str) -> Result<Self> {
    let normalized = normalize_abs_path(raw, PathFlavor::HOST)?;
    Ok(Self::from_normalized(&normalized))
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
  pub(crate) fn from_normalized(normalized: &str) -> Self {
    let bytes = normalized.as_bytes();
    Self(InternedSlice::new(fx_hash_bytes(bytes), bytes))
  }

  #[inline]
  pub fn as_str(&self) -> &str {
    // SAFETY: `AbsPath` only ever interns valid UTF-8 `&str` bytes.
    unsafe { std::str::from_utf8_unchecked(self.0.items()) }
  }

  #[inline]
  pub fn as_std_path(&self) -> &Path {
    Path::new(self.as_str())
  }

  #[inline]
  pub fn as_utf8_path(&self) -> &camino::Utf8Path {
    camino::Utf8Path::new(self.as_str())
  }

  #[inline]
  pub fn to_utf8_path_buf(&self) -> camino::Utf8PathBuf {
    camino::Utf8PathBuf::from(self.as_str())
  }

  #[inline]
  pub fn to_std_path_buf(&self) -> PathBuf {
    PathBuf::from(self.as_str())
  }

  #[inline]
  pub fn file_name(&self) -> Option<&str> {
    self.as_utf8_path().file_name()
  }

  #[inline]
  pub fn extension(&self) -> Option<&str> {
    self.as_utf8_path().extension()
  }

  #[inline]
  pub fn precomputed_hash(&self) -> u64 {
    *self.0.header()
  }

  pub fn parent(&self) -> Option<Self> {
    abs_parent(self.as_str(), PathFlavor::HOST).map(Self::from_normalized)
  }

  pub fn join(&self, sub: &str) -> Result<Self> {
    if sub.is_empty() {
      return Ok(self.clone());
    }
    let joined = abs_join(self.as_str(), sub, PathFlavor::HOST)?;
    Ok(Self::from_normalized(&joined))
  }

  #[inline]
  pub(crate) fn join_normalized_posix_rel(&self, rel: &str) -> Self {
    if rel.is_empty() {
      return self.clone();
    }
    let base = self.as_str().as_bytes();
    let sep = PathFlavor::HOST.separator() as u8;
    let needs_sep = !base.ends_with(&[sep]);
    let total_len = base.len() + usize::from(needs_sep) + rel.len();

    let copy_rel = |dst: &mut [u8]| {
      if cfg!(windows) {
        for (d, &b) in dst.iter_mut().zip(rel.as_bytes()) {
          *d = if b == b'/' { b'\\' } else { b };
        }
      } else {
        dst.copy_from_slice(rel.as_bytes());
      }
    };

    if total_len <= 512 {
      let mut buf = [0u8; 512];
      buf[..base.len()].copy_from_slice(base);
      let mut idx = base.len();
      if needs_sep {
        buf[idx] = sep;
        idx += 1;
      }
      copy_rel(&mut buf[idx..total_len]);
      let slice = &buf[..total_len];
      return Self(InternedSlice::new(fx_hash_bytes(slice), slice));
    }

    let mut buf = vec![0u8; total_len];
    buf[..base.len()].copy_from_slice(base);
    let mut idx = base.len();
    if needs_sep {
      buf[idx] = sep;
      idx += 1;
    }
    copy_rel(&mut buf[idx..total_len]);
    Self(InternedSlice::new(fx_hash_bytes(&buf), &buf))
  }

  pub fn strip_prefix(&self, base: &Self) -> Option<&str> {
    if self == base {
      return Some("");
    }
    abs_strip_prefix(self.as_str(), base.as_str(), PathFlavor::HOST)
  }
}

impl fmt::Debug for AbsPath {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.as_str().fmt(f)
  }
}

impl fmt::Display for AbsPath {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}

impl Hash for AbsPath {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(self.precomputed_hash());
  }
}

impl PartialOrd for AbsPath {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for AbsPath {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_str().cmp(other.as_str())
  }
}

impl AsRef<str> for AbsPath {
  #[inline]
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl AsRef<Path> for AbsPath {
  #[inline]
  fn as_ref(&self) -> &Path {
    self.as_std_path()
  }
}

impl AsRef<camino::Utf8Path> for AbsPath {
  #[inline]
  fn as_ref(&self) -> &camino::Utf8Path {
    self.as_utf8_path()
  }
}

impl std::str::FromStr for AbsPath {
  type Err = Error;

  #[inline]
  fn from_str(value: &str) -> Result<Self> {
    Self::new(value)
  }
}

impl TryFrom<&str> for AbsPath {
  type Error = Error;

  #[inline]
  fn try_from(value: &str) -> Result<Self> {
    Self::new(value)
  }
}

impl TryFrom<&camino::Utf8Path> for AbsPath {
  type Error = Error;

  #[inline]
  fn try_from(value: &camino::Utf8Path) -> Result<Self> {
    Self::try_from_utf8_path(value)
  }
}

impl TryFrom<&camino::Utf8PathBuf> for AbsPath {
  type Error = Error;

  #[inline]
  fn try_from(value: &camino::Utf8PathBuf) -> Result<Self> {
    Self::try_from_utf8_path(value.as_path())
  }
}

impl TryFrom<camino::Utf8PathBuf> for AbsPath {
  type Error = Error;

  #[inline]
  fn try_from(value: camino::Utf8PathBuf) -> Result<Self> {
    Self::try_from_utf8_path(value.as_path())
  }
}

impl TryFrom<&Path> for AbsPath {
  type Error = Error;

  #[inline]
  fn try_from(value: &Path) -> Result<Self> {
    Self::try_from_std_path(value)
  }
}

impl TryFrom<&PathBuf> for AbsPath {
  type Error = Error;

  #[inline]
  fn try_from(value: &PathBuf) -> Result<Self> {
    Self::try_from_std_path(value.as_path())
  }
}

impl TryFrom<PathBuf> for AbsPath {
  type Error = Error;

  #[inline]
  fn try_from(value: PathBuf) -> Result<Self> {
    Self::try_from_std_path(value.as_path())
  }
}

impl From<AbsPath> for camino::Utf8PathBuf {
  #[inline]
  fn from(value: AbsPath) -> Self {
    value.to_utf8_path_buf()
  }
}

impl From<&AbsPath> for PathBuf {
  #[inline]
  fn from(value: &AbsPath) -> Self {
    value.to_std_path_buf()
  }
}

impl From<AbsPath> for PathBuf {
  #[inline]
  fn from(value: AbsPath) -> Self {
    value.to_std_path_buf()
  }
}
