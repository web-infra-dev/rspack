#[cfg(windows)]
use std::borrow::Cow;
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
use indexmap::{IndexMap, IndexSet};
#[cfg(feature = "cacheable")]
use rspack_cacheable::cacheable;
use rspack_intern::{InternSliceStorage, InternedSlice, SliceInternable};
use rustc_hash::FxHasher;
pub use ustr::IdentityHasher;

#[cfg(feature = "cacheable")]
mod cacheable;
mod normalize;

pub use normalize::{normalize_native_path_separators, normalize_path_separators};

/// Returns the byte index immediately after a DOS device path prefix
/// (`\\\\?\\` or `\\\\.\\`), or zero when `path` has no such prefix.
///
/// The prefix is ASCII, so the byte index is also a valid UTF-8 slice boundary.
#[cfg(windows)]
#[inline]
pub fn dos_device_path_prefix_len(path: &str) -> usize {
  let bytes = path.as_bytes();
  if bytes.len() >= 4
    && bytes[0] == b'\\'
    && bytes[1] == b'\\'
    && matches!(bytes[2], b'?' | b'.')
    && bytes[3] == b'\\'
  {
    4
  } else {
    0
  }
}

/// Converts a DOS device drive or UNC path to its legacy Win32 spelling.
/// Other DOS device namespaces are returned unchanged.
#[cfg(windows)]
#[inline]
pub fn strip_dos_device_path_prefix(path: &str) -> Cow<'_, str> {
  let prefix_len = dos_device_path_prefix_len(path);
  if prefix_len == 0 {
    return Cow::Borrowed(path);
  }

  let path_without_prefix = &path[prefix_len..];
  let bytes = path_without_prefix.as_bytes();
  if bytes.len() >= 3
    && bytes[0].is_ascii_alphabetic()
    && bytes[1] == b':'
    && matches!(bytes[2], b'/' | b'\\')
  {
    return Cow::Borrowed(path_without_prefix);
  }

  if bytes
    .get(..4)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"UNC\\"))
  {
    let mut legacy_path = String::with_capacity(path_without_prefix.len() - 2);
    legacy_path.push_str(r"\\");
    legacy_path.push_str(&path_without_prefix[4..]);
    return Cow::Owned(legacy_path);
  }

  Cow::Borrowed(path)
}

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
/// Callers pass complete `as_encoded_bytes` slices from the same Rust version and target platform,
/// including native cache round trips.
#[inline]
fn path_from_bytes(bytes: &[u8]) -> &Path {
  // SAFETY: See above.
  Path::new(unsafe { OsStr::from_encoded_bytes_unchecked(bytes) })
}

/// An interned path: equal paths share one allocation process-wide, so equality is a pointer
/// comparison and each path is stored once. Hashing still uses the precomputed content hash
/// (see [`Hash`] below).
#[cfg_attr(feature = "cacheable", cacheable(with=cacheable::AsInternedPath))]
#[derive(Clone, PartialEq, Eq)]
pub struct InternedPath(InternedSlice<PreHashedPath>);

impl Debug for InternedPath {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_path().fmt(f)
  }
}

impl InternedPath {
  #[inline]
  pub fn new(path: &Path) -> Self {
    Self::from_bytes(path.as_os_str().as_encoded_bytes())
  }

  /// Intern complete `OsStr::as_encoded_bytes` from the same Rust version and target platform.
  #[inline]
  pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
    Self(InternedSlice::new(hash_path_bytes(bytes), bytes))
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
  hash_path_bytes(path.as_os_str().as_encoded_bytes())
}

#[inline]
fn hash_path_bytes(bytes: &[u8]) -> u64 {
  let mut hasher = FxHasher::default();
  #[cfg(unix)]
  hasher.write(bytes);
  #[cfg(not(unix))]
  path_from_bytes(bytes).hash(&mut hasher);
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

/// An indexed map using the path's precomputed hash.
pub type InternedPathIndexMap<V> = IndexMap<InternedPath, V, BuildHasherDefault<IdentityHasher>>;

/// A standard `IndexSet` using `InternedPath` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type InternedPathIndexSet = IndexSet<InternedPath, BuildHasherDefault<IdentityHasher>>;
