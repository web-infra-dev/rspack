use std::{
  borrow::Borrow,
  ops::{Deref, DerefMut},
  path::{Path, PathBuf},
  sync::Arc,
};

pub use camino::{Utf8Component, Utf8Components, Utf8Path, Utf8PathBuf, Utf8Prefix};
use rspack_cacheable::{
  cacheable,
  with::{AsRefStr, AsRefStrConverter},
};

pub trait AssertUtf8 {
  type Output;
  fn assert_utf8(self) -> Self::Output;
}

impl AssertUtf8 for PathBuf {
  type Output = Utf8PathBuf;

  /// Assert `self` is a valid UTF-8 [`PathBuf`] and convert to [`Utf8PathBuf`]
  ///
  /// # Panics
  ///
  /// Panics if `self` is not a valid UTF-8 path.
  fn assert_utf8(self) -> Self::Output {
    Utf8PathBuf::from_path_buf(self).unwrap_or_else(|p| {
      panic!("expected UTF-8 path, got: {}", p.display());
    })
  }
}

impl<'a> AssertUtf8 for &'a Path {
  type Output = &'a Utf8Path;

  /// Assert `self` is a valid UTF-8 [`Path`] and convert to [`Utf8Path`]
  ///
  /// # Panics
  ///
  /// Panics if `self` is not a valid UTF-8 path.
  fn assert_utf8(self) -> Self::Output {
    Utf8Path::from_path(self).unwrap_or_else(|| {
      panic!("expected UTF-8 path, got: {}", self.display());
    })
  }
}

#[cacheable(with=AsRefStr, hashable)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ArcPath(Arc<Path>);

impl Deref for ArcPath {
  type Target = Arc<Path>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for ArcPath {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<PathBuf> for ArcPath {
  fn from(value: PathBuf) -> Self {
    ArcPath(value.into())
  }
}

impl From<&Path> for ArcPath {
  fn from(value: &Path) -> Self {
    ArcPath(value.into())
  }
}

impl From<&Utf8Path> for ArcPath {
  fn from(value: &Utf8Path) -> Self {
    ArcPath(value.as_std_path().into())
  }
}

impl From<&ArcPath> for ArcPath {
  fn from(value: &ArcPath) -> Self {
    value.clone()
  }
}

impl Borrow<Path> for ArcPath {
  fn borrow(&self) -> &Path {
    &self.0
  }
}

impl AsRefStrConverter for ArcPath {
  fn as_str(&self) -> &str {
    self.0.to_str().expect("expect utf8 str")
  }
  fn from_str(s: &str) -> Self {
    Self::from(Path::new(s))
  }
}

fn _assert_size() {
  use std::mem::size_of;
  assert_eq!(size_of::<ArcPath>(), size_of::<[usize; 2]>());
}
