#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::{
  borrow::Cow,
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

#[inline]
fn is_path_separator(byte: u8) -> bool {
  byte == b'/' || byte == b'\\'
}

#[inline]
fn is_windows_drive_letter(byte: u8) -> bool {
  byte.is_ascii_alphabetic()
}

#[inline]
fn is_windows_drive_absolute_path(input: &str) -> bool {
  let bytes = input.as_bytes();
  bytes.len() >= 3
    && is_windows_drive_letter(bytes[0])
    && bytes[1] == b':'
    && is_path_separator(bytes[2])
}

#[inline]
fn is_windows_unc_path(input: &str) -> bool {
  let bytes = input.as_bytes();
  bytes.len() >= 2 && is_path_separator(bytes[0]) && is_path_separator(bytes[1])
}

#[inline]
fn is_windows_absolute_path(input: &str) -> bool {
  is_windows_drive_absolute_path(input) || is_windows_unc_path(input)
}

#[inline]
fn is_absolute_path(input: &str) -> bool {
  input.as_bytes().first() == Some(&b'/') || is_windows_absolute_path(input)
}

fn starts_with_url_scheme(input: &str) -> bool {
  let Some((scheme, _)) = input.split_once(':') else {
    return false;
  };
  let mut chars = scheme.chars();
  matches!(chars.next(), Some(c) if c.is_ascii_alphabetic())
    && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

fn starts_with_windows_drive_source_reference(input: &str) -> bool {
  let bytes = input.as_bytes();
  bytes.len() >= 2
    && is_windows_drive_letter(bytes[0])
    && matches!(bytes[1], b':' | b'|')
    && (bytes.len() == 2 || matches!(bytes[2], b'/' | b'\\' | b'?' | b'#'))
}

#[cacheable(with=Custom)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RspackPath {
  Absolute(Arc<Url>),
  Relative(SmolStr),
}

impl RspackPath {
  #[inline]
  pub fn is_absolute_request(input: &str) -> bool {
    is_absolute_path(input)
  }

  #[inline]
  pub fn is_path_separator_byte(byte: u8) -> bool {
    is_path_separator(byte)
  }

  #[inline]
  pub fn is_windows_drive_letter_byte(byte: u8) -> bool {
    is_windows_drive_letter(byte)
  }

  #[inline]
  pub fn is_windows_drive_scheme_guard(input: &str, colon_index: usize) -> bool {
    if colon_index != 1 {
      return false;
    }
    let Some(next) = input[colon_index + 1..].chars().next() else {
      return true;
    };
    next.is_ascii() && is_path_separator(next as u8) || matches!(next, '#' | '?')
  }

  pub fn is_source_map_relative_url_reference(input: &str) -> bool {
    !input.is_empty()
      && !input.starts_with(['/', '\\'])
      && !starts_with_url_scheme(input)
      && !starts_with_windows_drive_source_reference(input)
  }

  pub fn from_path_str(path: &str) -> Result<Self, String> {
    if is_absolute_path(path) {
      RspackPath::from_request(path, None)
    } else {
      Ok(RspackPath::Relative(path.into()))
    }
  }

  pub fn from_glob_pattern(pattern: &str) -> Self {
    let mut result = String::with_capacity(pattern.len());
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
      if c == '\\' {
        if chars
          .peek()
          .is_some_and(|next| matches!(next, '*' | '?' | '[' | ']' | '{' | '}'))
        {
          result.push(c);
        } else {
          result.push('/');
        }
      } else {
        result.push(c);
      }
    }
    Self::Relative(result.into())
  }

  pub fn join_glob_pattern(&self, child: &RspackPath) -> Self {
    let base = self.to_request_string();
    let child = child.to_request_string();
    let joined = if base.is_empty() {
      child
    } else if child.is_empty() {
      base
    } else if base.ends_with('/') || child.starts_with('/') {
      format!("{base}{child}")
    } else {
      format!("{base}/{child}")
    };
    Self::Relative(normalize_posix_path(&joined, joined.starts_with('/')).into())
  }

  pub fn join_request(&self, child: &str) -> Result<Self, String> {
    let child_path = RspackPath::from_request(child, None)?;
    if matches!(child_path, RspackPath::Absolute(_)) {
      return Ok(child_path);
    }

    if let RspackPath::Absolute(base) = self {
      let mut base = Url::clone(base);
      if !base.path().ends_with('/') {
        let path = format!("{}/", base.path());
        base.set_path(&path);
      }
      return base
        .join(child)
        .map(|url| RspackPath::Absolute(Arc::new(url)))
        .map_err(|err| format!("failed to join request: {err}"));
    }

    let base = self.to_request_path_string();
    let child = child_path.to_request_path_string();
    let joined = if base.is_empty() {
      child
    } else if child.is_empty() {
      base
    } else if base.ends_with('/') || child.starts_with('/') {
      format!("{base}{child}")
    } else {
      format!("{base}/{child}")
    };
    RspackPath::from_path_str(&normalize_request_path_string(&joined))
  }

  pub fn from_utf8_path(path: &Utf8Path) -> Result<Self, String> {
    Url::from_file_path(path.as_std_path())
      .map(Arc::new)
      .map(Self::Absolute)
      .map_err(|_| format!("failed to convert path to file URL: {path}"))
  }

  pub fn from_request(input: &str, base: Option<&RspackPath>) -> Result<Self, String> {
    parse_path(input, base)
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
      Self::Absolute(url) => Some(url.as_ref()),
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

  pub fn to_request_path_string(&self) -> String {
    match self {
      Self::Absolute(url) if url.scheme() == "file" => file_url_to_request_path(url),
      Self::Absolute(url) => url_to_request_path(url),
      Self::Relative(path) if path.as_bytes().contains(&b'\\') => path.replace('\\', "/"),
      Self::Relative(path) => path.to_string(),
    }
  }

  pub fn to_request_relative_to_context(&self, context: &RspackPath) -> Option<String> {
    let resource = self.to_request_path_string();
    let context = context.to_request_path_string();
    let relative = relative_request_path(&context, &resource)?;
    Some(relative_path_to_request(&relative))
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

fn parse_path(input: &str, base: Option<&RspackPath>) -> Result<RspackPath, String> {
  if let Some(url) = windows_file_url(input) {
    return Ok(RspackPath::Absolute(Arc::new(url)));
  }

  if let Ok(url) = Url::parse(input) {
    return Ok(RspackPath::Absolute(Arc::new(url)));
  }

  if let Some(base) = base.and_then(RspackPath::as_url)
    && let Ok(url) = base.join(input)
  {
    return Ok(RspackPath::Absolute(Arc::new(url)));
  }

  let (path, query, fragment) = split_resource(input);
  let path = Utf8Path::new(path);
  if path.is_absolute() {
    let mut url = Url::from_file_path(path.as_std_path())
      .map_err(|_| format!("failed to convert path to file URL: {path}"))?;
    if let Some(query) = query {
      url.set_query(Some(query.trim_start_matches('?')));
    }
    if let Some(fragment) = fragment {
      url.set_fragment(Some(fragment.trim_start_matches('#')));
    }
    return Ok(RspackPath::Absolute(Arc::new(url)));
  }

  Ok(RspackPath::Relative(input.into()))
}

fn windows_file_url(input: &str) -> Option<Url> {
  let normalized = input.replace('\\', "/");
  if is_windows_unc_path(input) {
    let mut parts = normalized.trim_start_matches('/').splitn(3, '/');
    let host = parts.next()?;
    let share = parts.next()?;
    let rest = parts.next().unwrap_or_default();
    return Url::parse(&format!("file://{host}/{share}/{rest}")).ok();
  }

  if is_windows_drive_absolute_path(input) {
    return Url::parse(&format!("file:///{}", normalized)).ok();
  }

  None
}

fn normalize_posix_path(path: &str, is_absolute: bool) -> String {
  let trailing_slash = path.ends_with('/');
  let mut parts = Vec::new();

  for part in path.split('/') {
    match part {
      "" | "." => {}
      ".." if parts.last().is_some_and(|last| *last != "..") => {
        parts.pop();
      }
      ".." if !is_absolute => parts.push(part),
      ".." => {}
      _ => parts.push(part),
    }
  }

  let mut normalized = parts.join("/");
  if is_absolute {
    normalized.insert(0, '/');
  }
  if trailing_slash && !normalized.ends_with('/') {
    normalized.push('/');
  }
  if normalized.is_empty() {
    if is_absolute {
      "/".to_string()
    } else {
      ".".to_string()
    }
  } else {
    normalized
  }
}

fn normalize_request_path_string(path: &str) -> String {
  if let Some((root, tail)) = split_request_root(path) {
    let tail = normalize_posix_path(tail, false);
    if tail == "." {
      root
    } else if root.ends_with('/') {
      format!("{root}{tail}")
    } else {
      format!("{root}/{tail}")
    }
  } else {
    normalize_posix_path(path, false)
  }
}

fn relative_path_to_request(relative: &str) -> String {
  if relative.is_empty() {
    "./.".to_string()
  } else if relative == ".." {
    "../.".to_string()
  } else if relative.starts_with("../") {
    relative.to_string()
  } else {
    format!("./{relative}")
  }
}

fn relative_request_path(context: &str, resource: &str) -> Option<String> {
  let (context_root, context_tail) = split_request_root(context)?;
  let (resource_root, resource_tail) = split_request_root(resource)?;
  if !context_root.eq_ignore_ascii_case(&resource_root) {
    return None;
  }

  let context_parts = path_parts(context_tail);
  let resource_parts = path_parts(resource_tail);
  let mut common = 0;
  while common < context_parts.len()
    && common < resource_parts.len()
    && context_parts[common].eq_ignore_ascii_case(resource_parts[common])
  {
    common += 1;
  }

  let mut relative_parts = Vec::with_capacity(context_parts.len() + resource_parts.len() - common);
  relative_parts.extend(std::iter::repeat_n("..", context_parts.len() - common));
  relative_parts.extend(resource_parts[common..].iter().copied());

  Some(if relative_parts.is_empty() {
    ".".to_string()
  } else {
    relative_parts.join("/")
  })
}

fn split_request_root(path: &str) -> Option<(String, &str)> {
  if let Some((drive, rest)) = windows_drive(path) {
    return Some((format!("{}:", drive as char), rest));
  }
  if path.starts_with("//") {
    let without_prefix = path.trim_start_matches('/');
    let mut parts = without_prefix.splitn(3, '/');
    let host = parts.next()?;
    let share = parts.next()?;
    let rest = parts.next().unwrap_or_default();
    return Some((format!("//{host}/{share}"), rest));
  }
  path.strip_prefix('/').map(|rest| ("/".to_string(), rest))
}

fn windows_drive(path: &str) -> Option<(u8, &str)> {
  let bytes = path.as_bytes();
  if bytes.len() >= 3 && is_windows_drive_letter(bytes[0]) && bytes[1] == b':' && bytes[2] == b'/' {
    Some((bytes[0], &path[3..]))
  } else {
    None
  }
}

fn path_parts(path: &str) -> Vec<&str> {
  path
    .trim_matches('/')
    .split('/')
    .filter(|part| !part.is_empty())
    .collect()
}

fn url_to_request_path(url: &Url) -> String {
  let mut url = url.clone();
  url.set_query(None);
  url.set_fragment(None);
  url.to_string()
}

fn file_url_to_request_path(url: &Url) -> String {
  if let Ok(path) = url.to_file_path() {
    let path = path.to_string_lossy();
    let path = if path.len() >= 4
      && path.as_bytes()[0] == b'/'
      && path.as_bytes()[1].is_ascii_alphabetic()
      && path.as_bytes()[2] == b':'
      && path.as_bytes()[3] == b'/'
    {
      Cow::Borrowed(&path[1..])
    } else {
      Cow::Borrowed(path.as_ref())
    };
    let path = if path.as_bytes().contains(&b'\\') {
      path.replace('\\', "/")
    } else {
      path.into_owned()
    };
    return normalize_request_path_string(&path);
  }

  let path = url.path();
  let path = if path.len() >= 4
    && path.as_bytes()[0] == b'/'
    && path.as_bytes()[1].is_ascii_alphabetic()
    && path.as_bytes()[2] == b':'
    && path.as_bytes()[3] == b'/'
  {
    &path[1..]
  } else {
    path
  };
  let path = normalize_request_path_string(path);
  if path.len() >= 4
    && path.as_bytes()[0] == b'/'
    && path.as_bytes()[1].is_ascii_alphabetic()
    && path.as_bytes()[2] == b':'
    && path.as_bytes()[3] == b'/'
  {
    path[1..].to_string()
  } else if let Some(host) = url.host_str()
    && !host.is_empty()
    && host != "localhost"
  {
    format!("//{host}{path}")
  } else {
    path.to_string()
  }
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
