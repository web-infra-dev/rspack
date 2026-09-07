#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::{
  collections::{HashMap, HashSet},
  ffi::OsStr,
  fmt::Debug,
  hash::{BuildHasherDefault, Hash, Hasher},
  ops::Deref,
  path::{Path, PathBuf},
};

pub use camino::{Utf8Component, Utf8Components, Utf8Path, Utf8PathBuf, Utf8Prefix};
use dashmap::{DashMap, DashSet};
use indexmap::IndexSet;
#[cfg(feature = "cacheable")]
use rspack_cacheable::{
  ContextGuard, Error as CacheableError, cacheable,
  utils::PortablePath,
  with::{Custom, CustomConverter},
};
use rspack_intern::{InternSliceStorage, InternedSlice, SliceInternable};
use rustc_hash::FxHasher;
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

/// The interning kind for paths: a header holding the precomputed [`hash_path`] of the path,
/// plus the path's `OsStr` bytes, stored together in one allocation.
///
/// This is what makes an [`InternedPath`] a thin pointer with a content hash attached and no second
/// indirection to reach the bytes.
pub struct PreHashedPath;

impl SliceInternable for PreHashedPath {
  type Header = u64;
  type Item = u8;

  /// The header already is the path's hash, so interning never rehashes a path.
  #[inline]
  fn hash(header: &u64, _bytes: &[u8]) -> u64 {
    *header
  }

  /// Mirrors [`hash_path`]'s per-platform scheme, so that two paths which hash the same are
  /// deduplicated exactly when they compare equal: raw `OsStr` bytes on Unix (matching the
  /// bulk-byte hash), component-normalized `Path::eq` elsewhere (matching `Path::hash`).
  fn eq(a: &[u8], b: &[u8]) -> bool {
    #[cfg(unix)]
    {
      a == b
    }
    #[cfg(not(unix))]
    {
      // Identical bytes are identical paths, and that is what nearly every probe sees. Only
      // spellings that differ but normalize to the same components pay for `Path::components`.
      a == b || path_from_bytes(a) == path_from_bytes(b)
    }
  }

  fn storage() -> &'static InternSliceStorage<Self> {
    static STORAGE: InternSliceStorage<PreHashedPath> = InternSliceStorage::new();
    &STORAGE
  }
}

/// Reads back bytes produced by `OsStr::as_encoded_bytes`.
///
/// # Panics-free safety
/// The bytes always come from [`InternedPath::from_parts`], which takes them from `as_encoded_bytes`
/// on this same platform and stores the whole slice — the round trip `OsStr` documents as sound.
#[inline]
fn path_from_bytes(bytes: &[u8]) -> &Path {
  // SAFETY: See above.
  Path::new(unsafe { OsStr::from_encoded_bytes_unchecked(bytes) })
}

/// An interned path: equal paths share one allocation process-wide, so equality is a pointer
/// comparison and each path is stored once. Hashing still uses the precomputed content hash
/// (see [`Hash`] below).
#[cfg_attr(feature = "cacheable", cacheable(with=Custom))]
#[derive(Clone, PartialEq, Eq)]
pub struct InternedPath(InternedSlice<PreHashedPath>);

impl Debug for InternedPath {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_path().fmt(f)
  }
}

impl InternedPath {
  pub fn new(path: &Path) -> Self {
    Self::from_parts(hash_path(path), path)
  }

  /// Build an `InternedPath` from a precomputed hash without rehashing. The caller MUST guarantee
  /// that `hash` equals [`hash_path`] of `path`. Used at boundaries (e.g. consuming
  /// `rspack_resolver::ResolverPath`) where the same `FxHash` has already been computed
  /// upstream.
  #[inline]
  pub fn from_parts(hash: u64, path: &Path) -> Self {
    Self(InternedSlice::new(
      hash,
      path.as_os_str().as_encoded_bytes(),
    ))
  }

  #[inline]
  pub fn as_path(&self) -> &Path {
    path_from_bytes(self.0.items())
  }

  /// The `FxHash` of the path bytes, computed once when the path was first interned.
  #[inline]
  pub fn precomputed_hash(&self) -> u64 {
    *self.0.header()
  }
}

/// Hash a path with `FxHasher`, hashing the raw `OsStr` bytes on Unix rather than walking
/// components as `Path::hash` does — materially cheaper on the resolver's hot path.
///
/// `rspack_resolver` computes this once per cached path, which lets [`InternedPath::from_parts`]
/// intern a resolved dependency without rehashing it.
#[inline]
pub fn hash_path(path: &Path) -> u64 {
  let mut hasher = FxHasher::default();
  #[cfg(unix)]
  hasher.write(path.as_os_str().as_bytes());
  #[cfg(not(unix))]
  path.hash(&mut hasher);
  hasher.finish()
}

impl Deref for InternedPath {
  type Target = Path;

  fn deref(&self) -> &Self::Target {
    self.as_path()
  }
}

impl AsRef<Path> for InternedPath {
  fn as_ref(&self) -> &Path {
    self.as_path()
  }
}

impl From<PathBuf> for InternedPath {
  fn from(value: PathBuf) -> Self {
    InternedPath::new(&value)
  }
}

impl From<&PathBuf> for InternedPath {
  fn from(value: &PathBuf) -> Self {
    InternedPath::new(value)
  }
}

impl From<&Path> for InternedPath {
  fn from(value: &Path) -> Self {
    InternedPath::new(value)
  }
}

impl From<Utf8PathBuf> for InternedPath {
  fn from(value: Utf8PathBuf) -> Self {
    InternedPath::new(value.as_std_path())
  }
}

impl From<&Utf8Path> for InternedPath {
  fn from(value: &Utf8Path) -> Self {
    InternedPath::new(value.as_std_path())
  }
}

impl From<&InternedPath> for InternedPath {
  fn from(value: &InternedPath) -> Self {
    value.clone()
  }
}

impl From<&str> for InternedPath {
  fn from(value: &str) -> Self {
    InternedPath::new(<str as std::convert::AsRef<Path>>::as_ref(value))
  }
}

#[cfg(feature = "cacheable")]
impl CustomConverter for InternedPath {
  type Target = PortablePath;
  fn serialize(&self, guard: &ContextGuard) -> Result<Self::Target, CacheableError> {
    Ok(PortablePath::new(self.as_path(), guard.project_root()))
  }
  fn deserialize(data: Self::Target, guard: &ContextGuard) -> Result<Self, CacheableError> {
    Ok(Self::from(PathBuf::from(
      data.into_path_string(guard.project_root()),
    )))
  }
}

impl Hash for InternedPath {
  /// Hashes by content, not by the interned pointer: [`InternedPathMap`] and friends feed this
  /// straight into [`IdentityHasher`], and pointer addresses are allocation-aligned (low bits
  /// always zero, so hashbrown would cluster every entry into a few buckets) and differ between
  /// runs, which would make anything ordered by hash non-deterministic.
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(self.precomputed_hash());
  }
}

/// A standard `HashMap` using `InternedPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type InternedPathMap<V> = HashMap<InternedPath, V, BuildHasherDefault<IdentityHasher>>;

/// A standard `HashSet` using `InternedPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type InternedPathSet = HashSet<InternedPath, BuildHasherDefault<IdentityHasher>>;

/// A standard `DashMap` using `InternedPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type InternedPathDashMap<V> = DashMap<InternedPath, V, BuildHasherDefault<IdentityHasher>>;

/// A standard `DashSet` using `InternedPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type InternedPathDashSet = DashSet<InternedPath, BuildHasherDefault<IdentityHasher>>;

/// A standard `IndexSet` using `InternedPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type InternedPathIndexSet = IndexSet<InternedPath, BuildHasherDefault<IdentityHasher>>;
