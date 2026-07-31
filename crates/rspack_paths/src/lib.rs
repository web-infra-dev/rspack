use std::{
  collections::{HashMap, HashSet},
  fmt::Debug,
  hash::BuildHasherDefault,
  ops::Deref,
  path::{Path, PathBuf},
};

pub use camino::{Utf8Component, Utf8Components, Utf8Path, Utf8PathBuf, Utf8Prefix};
use dashmap::{DashMap, DashSet};
use indexmap::IndexSet;
use rspack_cacheable::{
  ContextGuard, Error as CacheableError, cacheable,
  utils::PortablePath,
  with::{Custom, CustomConverter},
};
pub use rspack_resolver::{ToUstrPath, UstrPath, UstrPathSet};
pub use ustr::IdentityHasher;

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

/// An interned absolute path.
///
/// Backed by [`UstrPath`], so two `ArcPath`s naming the same path are the same
/// 8-byte handle: equality is an integer compare and hashing is a `write_u64`
/// of the hash the interner already computed. The name is kept for historical
/// reasons — there is no longer an `Arc` involved.
#[cacheable(with=Custom)]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ArcPath(UstrPath);

impl Debug for ArcPath {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // Forward to `Path` rather than `UstrPath` so the rendering stays identical
    // to the previous `Arc<Path>`-backed representation.
    self.0.as_std_path().fmt(f)
  }
}

impl ArcPath {
  pub fn new<P: ToUstrPath + ?Sized>(path: &P) -> Self {
    Self(path.to_ustr_path())
  }
}

impl Deref for ArcPath {
  type Target = Path;

  fn deref(&self) -> &Self::Target {
    self.0.as_std_path()
  }
}

impl AsRef<Path> for ArcPath {
  fn as_ref(&self) -> &Path {
    self.0.as_std_path()
  }
}

impl From<PathBuf> for ArcPath {
  fn from(value: PathBuf) -> Self {
    ArcPath::new(value.as_path())
  }
}

impl From<&Path> for ArcPath {
  fn from(value: &Path) -> Self {
    ArcPath::new(value)
  }
}

impl From<&Utf8Path> for ArcPath {
  fn from(value: &Utf8Path) -> Self {
    ArcPath::new(value)
  }
}

impl From<&ArcPath> for ArcPath {
  fn from(value: &ArcPath) -> Self {
    value.clone()
  }
}

impl From<&str> for ArcPath {
  fn from(value: &str) -> Self {
    ArcPath::new(value)
  }
}

impl From<UstrPath> for ArcPath {
  /// Free: both sides are the same interned 8-byte handle.
  #[inline]
  fn from(value: UstrPath) -> Self {
    Self(value)
  }
}

impl CustomConverter for ArcPath {
  type Target = PortablePath;
  fn serialize(&self, guard: &ContextGuard) -> Result<Self::Target, CacheableError> {
    Ok(PortablePath::new(
      self.0.as_std_path(),
      guard.project_root(),
    ))
  }
  fn deserialize(data: Self::Target, guard: &ContextGuard) -> Result<Self, CacheableError> {
    Ok(Self::from(PathBuf::from(
      data.into_path_string(guard.project_root()),
    )))
  }
}

/// A standard `HashMap` using `ArcPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type ArcPathMap<V> = HashMap<ArcPath, V, BuildHasherDefault<IdentityHasher>>;

/// A standard `HashSet` using `ArcPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type ArcPathSet = HashSet<ArcPath, BuildHasherDefault<IdentityHasher>>;

/// A `HashSet<UstrPath, IdentityHasher>` that preserves the `FxHash` the
/// interner stamped into the `UstrPath` handle. Inserting and looking up
/// entries here only costs a `write_u64` instead of hashing the full path.
pub type ArcResolverPathSet = UstrPathSet;

/// A standard `DashMap` using `ArcPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type ArcPathDashMap<V> = DashMap<ArcPath, V, BuildHasherDefault<IdentityHasher>>;

/// A standard `DashSet` using `ArcPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type ArcPathDashSet = DashSet<ArcPath, BuildHasherDefault<IdentityHasher>>;

/// A standard `IndexSet` using `ArcPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type ArcPathIndexSet = IndexSet<ArcPath, BuildHasherDefault<IdentityHasher>>;
