#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::{
  collections::{HashMap, HashSet},
  fmt::Debug,
  hash::{BuildHasherDefault, Hash, Hasher},
  ops::Deref,
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
pub use rspack_resolver::ResolverPath;
use rustc_hash::FxHasher;
use smol_str::SmolStr;
use url::Url;
pub use ustr::IdentityHasher;

pub trait AssertUtf8 {
  type Output;
  fn assert_utf8(self) -> Self::Output;
}

#[cacheable(with=Custom)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RspackPath {
  Absolute(Url),
  Relative(SmolStr),
}

impl RspackPath {
  pub fn from_utf8_path(path: &Utf8Path) -> Result<Self, String> {
    Url::from_file_path(path.as_std_path())
      .map(Self::Absolute)
      .map_err(|_| format!("failed to convert path to file URL: {path}"))
  }

  pub fn from_request(input: &str, base: Option<&RspackPath>) -> Result<Self, String> {
    let resource = RspackResource::from_request(input, base)?;
    Ok(resource.path)
  }

  pub fn as_file_path(&self) -> Option<Utf8PathBuf> {
    let Self::Absolute(url) = self else {
      return None;
    };
    if url.scheme() != "file" {
      return None;
    }
    let path = url.to_file_path().ok()?;
    Utf8PathBuf::from_path_buf(path).ok()
  }

  pub fn as_url(&self) -> Option<&Url> {
    match self {
      Self::Absolute(url) => Some(url),
      Self::Relative(_) => None,
    }
  }

  pub fn to_request_string(&self) -> String {
    match self {
      Self::Absolute(url) => url.to_string(),
      Self::Relative(path) => path.to_string(),
    }
  }

  pub fn to_display_path(&self, context: Option<&Utf8Path>) -> String {
    if let Some(path) = self.as_file_path() {
      if let Some(context) = context
        && let Ok(relative) = path.strip_prefix(context)
      {
        return relative.to_string();
      }
      return path.to_string();
    }
    self.to_request_string()
  }

  pub fn to_cache_key(&self) -> String {
    self.to_request_string()
  }
}

impl CustomConverter for RspackPath {
  type Target = String;

  fn serialize(&self, _guard: &ContextGuard) -> Result<Self::Target, CacheableError> {
    Ok(self.to_request_string())
  }

  fn deserialize(data: Self::Target, _guard: &ContextGuard) -> Result<Self, CacheableError> {
    Ok(RspackPath::from_request(&data, None).unwrap_or_else(|_| RspackPath::Relative(data.into())))
  }
}

#[cacheable(with=Custom)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RspackResource {
  pub path: RspackPath,
  pub query: Option<SmolStr>,
  pub fragment: Option<SmolStr>,
}

impl RspackResource {
  pub fn from_request(input: &str, base: Option<&RspackPath>) -> Result<Self, String> {
    let (path, query, fragment) = split_resource(input);
    let parsed_path = parse_path(path, base)?;
    Ok(Self {
      path: parsed_path,
      query: query.map(Into::into),
      fragment: fragment.map(Into::into),
    })
  }

  pub fn from_parts(
    path: RspackPath,
    query: Option<impl Into<SmolStr>>,
    fragment: Option<impl Into<SmolStr>>,
  ) -> Self {
    Self {
      path,
      query: query.map(Into::into),
      fragment: fragment.map(Into::into),
    }
  }

  pub fn as_file_path(&self) -> Option<Utf8PathBuf> {
    self.path.as_file_path()
  }

  pub fn as_url(&self) -> Option<&Url> {
    self.path.as_url()
  }

  pub fn to_request_string(&self) -> String {
    let mut resource = self.path.to_request_string();
    if let Some(query) = &self.query {
      resource.push_str(query);
    }
    if let Some(fragment) = &self.fragment {
      resource.push_str(fragment);
    }
    resource
  }

  pub fn to_cache_key(&self) -> String {
    self.to_request_string()
  }
}

impl CustomConverter for RspackResource {
  type Target = String;

  fn serialize(&self, _guard: &ContextGuard) -> Result<Self::Target, CacheableError> {
    Ok(self.to_request_string())
  }

  fn deserialize(data: Self::Target, _guard: &ContextGuard) -> Result<Self, CacheableError> {
    RspackResource::from_request(&data, None)
      .map_err(|_| CacheableError::MessageError("failed to deserialize RspackResource"))
  }
}

fn parse_path(input: &str, base: Option<&RspackPath>) -> Result<RspackPath, String> {
  if let Some(url) = windows_file_url(input) {
    return Ok(RspackPath::Absolute(url));
  }

  if let Ok(url) = Url::parse(input) {
    return Ok(RspackPath::Absolute(url));
  }

  if let Some(base) = base.and_then(RspackPath::as_url)
    && let Ok(url) = base.join(input)
  {
    return Ok(RspackPath::Absolute(url));
  }

  let path = Utf8Path::new(input);
  if path.is_absolute() {
    return RspackPath::from_utf8_path(path);
  }

  Ok(RspackPath::Relative(input.into()))
}

fn windows_file_url(input: &str) -> Option<Url> {
  let normalized = input.replace('\\', "/");
  if normalized.starts_with("//") {
    let mut parts = normalized.trim_start_matches('/').splitn(3, '/');
    let host = parts.next()?;
    let share = parts.next()?;
    let rest = parts.next().unwrap_or_default();
    return Url::parse(&format!("file://{host}/{share}/{rest}")).ok();
  }

  let bytes = normalized.as_bytes();
  if bytes.len() >= 3 && bytes[1] == b':' && bytes[2] == b'/' && bytes[0].is_ascii_alphabetic() {
    return Url::parse(&format!("file:///{}", normalized)).ok();
  }

  None
}

fn split_resource(input: &str) -> (&str, Option<&str>, Option<&str>) {
  let path_end = input.find(['?', '#']).unwrap_or(input.len());
  let path = &input[..path_end];
  let rest = &input[path_end..];

  match rest.as_bytes().first() {
    Some(b'?') => {
      let fragment_start = rest.find('#').unwrap_or(rest.len());
      let query = &rest[..fragment_start];
      let fragment = (fragment_start < rest.len()).then_some(&rest[fragment_start..]);
      (path, Some(query), fragment)
    }
    Some(b'#') => (path, None, Some(rest)),
    _ => (path, None, None),
  }
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
    let hash = hash_path(&path);
    Self { path, hash }
  }

  /// Build an `ArcPath` from a precomputed hash and an `Arc<Path>` without
  /// rehashing. The caller MUST guarantee that `hash` equals [`hash_path`] of
  /// `path`. Used at boundaries (e.g. consuming `rspack_resolver::ResolverPath`)
  /// where the same `FxHash` has already been computed upstream.
  #[inline]
  pub fn from_parts(hash: u64, path: Arc<Path>) -> Self {
    Self { path, hash }
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
  type Target = Arc<Path>;

  fn deref(&self) -> &Self::Target {
    &self.path
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

impl From<ResolverPath> for ArcPath {
  /// Zero-cost conversion: reuses the resolver's precomputed `FxHash` and the
  /// existing `Arc<Path>`. Safe because `rspack_paths::hash_path` and the hash
  /// scheme in `rspack_resolver` are kept identical.
  fn from(value: ResolverPath) -> Self {
    let hash = value.precomputed_hash();
    ArcPath::from_parts(hash, value.into_arc())
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
