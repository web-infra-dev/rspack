#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::{
  fmt,
  hash::{Hash, Hasher},
  ops::Deref,
  path::{Path, PathBuf},
  sync::Arc,
};

use camino::{Utf8Path, Utf8PathBuf};
use rustc_hash::FxHasher;

/// A path returned in [`crate::ResolveContext`] dependencies, paired with a
/// precomputed `FxHash` of the path bytes.
///
/// Downstream consumers (rspack) place these into hash collections keyed by
/// the precomputed hash, avoiding repeated hashing of long absolute paths on
/// every insert and lookup.
///
/// Hash and equality are kept aligned per platform so the standard
/// `a == b ⇒ hash(a) == hash(b)` contract holds:
/// - On Unix, both hash and equality compare the raw `OsStr` bytes (fast bulk
///   `write`; matches the resolver's cache-side `Cache::value` hash).
/// - On other platforms, both go through `Path` (component-walked hash and
///   normalized equality), so e.g. `pack1/foo` and `pack1\\foo` are treated as
///   the same dependency on Windows — same behavior `PathBuf` had before.
#[derive(Clone)]
pub struct ResolverPath {
  hash: u64,
  path: Arc<Path>,
}

impl ResolverPath {
  pub fn new(path: Arc<Path>) -> Self {
    let hash = hash_path(&path);
    Self { hash, path }
  }

  /// Construct without recomputing the hash.
  ///
  /// # Precondition
  /// `hash` MUST equal `hash_path(path)`. Violating this breaks `HashSet`'s
  /// bucketing invariant — entries become unfindable and deduplication stops
  /// working. Not `unsafe` because the failure mode is a logic bug rather
  /// than UB.
  #[inline]
  pub(crate) fn from_parts(hash: u64, path: Arc<Path>) -> Self {
    Self { hash, path }
  }

  #[inline]
  pub fn as_path(&self) -> &Path {
    &self.path
  }

  #[inline]
  pub fn as_arc(&self) -> &Arc<Path> {
    &self.path
  }

  #[inline]
  pub fn into_arc(self) -> Arc<Path> {
    self.path
  }

  /// The precomputed `FxHash` of the path bytes.
  #[inline]
  pub fn precomputed_hash(&self) -> u64 {
    self.hash
  }
}

/// Hash a path with `FxHasher`, matching the bytes-on-unix optimization used by
/// the resolver's internal cache so [`ResolverPath`] values constructed from a
/// `CachedPath` produce the same `u64` as values constructed from a `&Path`.
#[inline]
pub fn hash_path(path: &Path) -> u64 {
  let mut hasher = FxHasher::default();
  // The std `Path::hash` impl walks components (utf8 split + per-segment
  // write); a single bulk `write` of the raw bytes is materially cheaper on
  // the resolver hot path.
  #[cfg(unix)]
  hasher.write(path.as_os_str().as_bytes());
  #[cfg(not(unix))]
  path.hash(&mut hasher);
  hasher.finish()
}

impl Hash for ResolverPath {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(self.hash);
  }
}

impl PartialEq for ResolverPath {
  /// Mirror `hash_path`'s per-platform scheme so the `a == b ⇒ hash(a) ==
  /// hash(b)` invariant holds: raw `OsStr` bytes on Unix (matches the
  /// bulk-byte hash), component-normalized `Path::eq` elsewhere (matches
  /// `Path::hash`).
  fn eq(&self, other: &Self) -> bool {
    #[cfg(unix)]
    {
      self.path.as_os_str() == other.path.as_os_str()
    }
    #[cfg(not(unix))]
    {
      self.path == other.path
    }
  }
}

impl Eq for ResolverPath {}

impl Deref for ResolverPath {
  type Target = Path;

  fn deref(&self) -> &Self::Target {
    &self.path
  }
}

impl AsRef<Path> for ResolverPath {
  fn as_ref(&self) -> &Path {
    &self.path
  }
}

impl From<PathBuf> for ResolverPath {
  fn from(path: PathBuf) -> Self {
    Self::new(Arc::from(path))
  }
}

impl From<&Path> for ResolverPath {
  fn from(path: &Path) -> Self {
    Self::new(Arc::from(path))
  }
}

impl From<&PathBuf> for ResolverPath {
  fn from(path: &PathBuf) -> Self {
    Self::new(Arc::from(path.as_path()))
  }
}

impl From<Arc<Path>> for ResolverPath {
  fn from(path: Arc<Path>) -> Self {
    Self::new(path)
  }
}

// The resolver stores paths internally as UTF-8 (`camino`); these accept the
// internal representation directly while keeping the stored buffer an
// `Arc<Path>` so the `as_arc`/`into_arc` output contract stays unchanged.
impl From<Utf8PathBuf> for ResolverPath {
  fn from(path: Utf8PathBuf) -> Self {
    Self::new(Arc::from(path.into_std_path_buf()))
  }
}

impl From<&Utf8Path> for ResolverPath {
  fn from(path: &Utf8Path) -> Self {
    Self::new(Arc::from(path.as_std_path()))
  }
}

impl fmt::Debug for ResolverPath {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.path.fmt(f)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hash_is_path_byte_hash() {
    let p: &Path = Path::new("/a/b/c.js");
    let rp = ResolverPath::from(p);
    assert_eq!(rp.precomputed_hash(), hash_path(p));
  }

  #[test]
  fn equal_paths_have_equal_hashes() {
    let a = ResolverPath::from(PathBuf::from("/x/y"));
    let b = ResolverPath::from(Path::new("/x/y"));
    assert_eq!(a, b);
    assert_eq!(a.precomputed_hash(), b.precomputed_hash());
  }

  #[test]
  fn writes_u64_into_hasher() {
    use std::{collections::HashSet, hash::BuildHasherDefault};

    #[derive(Default)]
    struct IdHasher(u64);
    impl Hasher for IdHasher {
      fn write(&mut self, _: &[u8]) {
        unreachable!()
      }
      fn write_u64(&mut self, n: u64) {
        self.0 = n;
      }
      fn finish(&self) -> u64 {
        self.0
      }
    }

    let mut set: HashSet<ResolverPath, BuildHasherDefault<IdHasher>> = HashSet::default();
    set.insert(ResolverPath::from(Path::new("/a/b")));
    assert!(set.contains(&ResolverPath::from(PathBuf::from("/a/b"))));
  }
}
