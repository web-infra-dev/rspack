use std::{
  collections::{HashMap, HashSet},
  fmt::Debug,
  hash::{BuildHasherDefault, Hash, Hasher},
  ops::{Deref, DerefMut},
  path::{Path, PathBuf},
  sync::Arc,
};

pub use camino::{Utf8Component, Utf8Components, Utf8Path, Utf8PathBuf, Utf8Prefix};
use dashmap::{DashMap, DashSet};
use indexmap::IndexSet;
use rspack_cacheable::{
  ContextGuard, Error as CacheableError, cacheable,
  utils::PortablePath,
  with::{Custom, CustomConverter},
};
use rustc_hash::FxHasher;
use ustr::IdentityHasher;

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

#[cacheable(with=Custom)]
#[derive(Clone, PartialEq, Eq)]
pub struct ArcPath {
  path: Arc<Path>,
  // Pre-calculating and caching the hash value upon creation, making hashing operations
  // in collections virtually free.
  hash: u64,
}

impl Debug for ArcPath {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.path.fmt(f)
  }
}

impl ArcPath {
  pub fn new(path: Arc<Path>) -> Self {
    let mut hasher = FxHasher::default();
    path.hash(&mut hasher);
    let hash = hasher.finish();
    Self { path, hash }
  }
}

impl Deref for ArcPath {
  type Target = Arc<Path>;

  fn deref(&self) -> &Self::Target {
    &self.path
  }
}

impl DerefMut for ArcPath {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.path
  }
}

impl AsRef<Path> for ArcPath {
  fn as_ref(&self) -> &Path {
    &self.path
  }
}

impl From<PathBuf> for ArcPath {
  fn from(value: PathBuf) -> Self {
    ArcPath::new(value.into())
  }
}

impl From<&Path> for ArcPath {
  fn from(value: &Path) -> Self {
    ArcPath::new(value.into())
  }
}

impl From<&Utf8Path> for ArcPath {
  fn from(value: &Utf8Path) -> Self {
    ArcPath::new(value.as_std_path().into())
  }
}

impl From<&ArcPath> for ArcPath {
  fn from(value: &ArcPath) -> Self {
    value.clone()
  }
}

impl From<&str> for ArcPath {
  fn from(value: &str) -> Self {
    ArcPath::new(<str as std::convert::AsRef<Path>>::as_ref(value).into())
  }
}

impl CustomConverter for ArcPath {
  type Target = PortablePath;
  fn serialize(&self, guard: &ContextGuard) -> Result<Self::Target, CacheableError> {
    Ok(PortablePath::new(&self.path, guard.project_root()))
  }
  fn deserialize(data: Self::Target, guard: &ContextGuard) -> Result<Self, CacheableError> {
    Ok(Self::from(PathBuf::from(
      data.into_path_string(guard.project_root()),
    )))
  }
}

impl Hash for ArcPath {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(self.hash);
  }
}

/// A standard `HashMap` using `ArcPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type ArcPathMap<V> = HashMap<ArcPath, V, BuildHasherDefault<IdentityHasher>>;

/// A standard `HashSet` using `ArcPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type ArcPathSet = HashSet<ArcPath, BuildHasherDefault<IdentityHasher>>;

/// A standard `DashMap` using `ArcPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type ArcPathDashMap<V> = DashMap<ArcPath, V, BuildHasherDefault<IdentityHasher>>;

/// A standard `DashSet` using `ArcPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type ArcPathDashSet = DashSet<ArcPath, BuildHasherDefault<IdentityHasher>>;

/// A standard `IndexSet` using `ArcPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type ArcPathIndexSet = IndexSet<ArcPath, BuildHasherDefault<IdentityHasher>>;
