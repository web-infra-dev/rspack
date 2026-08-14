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
use rspack_cacheable::{
  ContextGuard, Error as CacheableError, cacheable,
  utils::PortablePath,
  with::{Custom, CustomConverter},
};
use rspack_intern::{InternSliceStorage, InternedSlice, SliceInternable};
pub use rspack_resolver::ResolverPath;
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
/// This is what makes an [`ArcPath`] a thin pointer with a content hash attached and no second
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
      path_from_bytes(a) == path_from_bytes(b)
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
/// The bytes always come from [`ArcPath::from_parts`], which takes them from `as_encoded_bytes`
/// on this same platform and stores the whole slice — the round trip `OsStr` documents as sound.
#[inline]
fn path_from_bytes(bytes: &[u8]) -> &Path {
  // SAFETY: See above.
  Path::new(unsafe { OsStr::from_encoded_bytes_unchecked(bytes) })
}

/// An interned path: equal paths share one allocation process-wide, so equality is a pointer
/// comparison and each path is stored once. Hashing still uses the precomputed content hash
/// (see [`Hash`] below).
#[cacheable(with=Custom)]
#[derive(Clone, PartialEq, Eq)]
pub struct ArcPath(InternedSlice<PreHashedPath>);

impl Debug for ArcPath {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_path().fmt(f)
  }
}

impl ArcPath {
  pub fn new(path: &Path) -> Self {
    Self::from_parts(hash_path(path), path)
  }

  /// Build an `ArcPath` from a precomputed hash without rehashing. The caller MUST guarantee
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

/// Hash a path with `FxHasher` matching the bytes-on-unix optimization used by
/// `rspack_resolver`. Keeping these in sync lets `ArcPath::from_parts` accept
/// a hash precomputed inside the resolver without rehashing here.
#[inline]
pub fn hash_path(path: &Path) -> u64 {
  let mut hasher = FxHasher::default();
  #[cfg(unix)]
  hasher.write(path.as_os_str().as_bytes());
  #[cfg(not(unix))]
  path.hash(&mut hasher);
  hasher.finish()
}

impl Deref for ArcPath {
  type Target = Path;

  fn deref(&self) -> &Self::Target {
    self.as_path()
  }
}

impl AsRef<Path> for ArcPath {
  fn as_ref(&self) -> &Path {
    self.as_path()
  }
}

impl From<PathBuf> for ArcPath {
  fn from(value: PathBuf) -> Self {
    ArcPath::new(&value)
  }
}

impl From<&Path> for ArcPath {
  fn from(value: &Path) -> Self {
    ArcPath::new(value)
  }
}

impl From<&Utf8Path> for ArcPath {
  fn from(value: &Utf8Path) -> Self {
    ArcPath::new(value.as_std_path())
  }
}

impl From<&ArcPath> for ArcPath {
  fn from(value: &ArcPath) -> Self {
    value.clone()
  }
}

impl From<&str> for ArcPath {
  fn from(value: &str) -> Self {
    ArcPath::new(<str as std::convert::AsRef<Path>>::as_ref(value))
  }
}

impl From<ResolverPath> for ArcPath {
  /// Reuses the resolver's precomputed `FxHash` instead of rehashing. Sound because
  /// `rspack_paths::hash_path` and the hash scheme in `rspack_resolver` are kept identical.
  fn from(value: ResolverPath) -> Self {
    ArcPath::from_parts(value.precomputed_hash(), value.as_path())
  }
}

impl CustomConverter for ArcPath {
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

impl Hash for ArcPath {
  /// Hashes by content, not by the interned pointer: [`ArcPathMap`] and friends feed this
  /// straight into [`IdentityHasher`], and pointer addresses are allocation-aligned (low bits
  /// always zero, so hashbrown would cluster every entry into a few buckets) and differ between
  /// runs, which would make anything ordered by hash non-deterministic.
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(self.precomputed_hash());
  }
}

/// A standard `HashMap` using `ArcPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type ArcPathMap<V> = HashMap<ArcPath, V, BuildHasherDefault<IdentityHasher>>;

/// A standard `HashSet` using `ArcPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type ArcPathSet = HashSet<ArcPath, BuildHasherDefault<IdentityHasher>>;

/// A `HashSet<ResolverPath, IdentityHasher>` that preserves the `FxHash`
/// precomputed inside `rspack_resolver`. Inserting and looking up entries
/// here only costs a `write_u64` instead of hashing the full absolute path.
pub type ArcResolverPathSet = HashSet<ResolverPath, BuildHasherDefault<IdentityHasher>>;

/// A standard `DashMap` using `ArcPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type ArcPathDashMap<V> = DashMap<ArcPath, V, BuildHasherDefault<IdentityHasher>>;

/// A standard `DashSet` using `ArcPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type ArcPathDashSet = DashSet<ArcPath, BuildHasherDefault<IdentityHasher>>;

/// A standard `IndexSet` using `ArcPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type ArcPathIndexSet = IndexSet<ArcPath, BuildHasherDefault<IdentityHasher>>;
