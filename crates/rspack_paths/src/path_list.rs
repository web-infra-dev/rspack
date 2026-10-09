use std::{
  fmt,
  hash::{Hash, Hasher},
  ops::Deref,
  sync::Arc,
};

#[cfg(feature = "cacheable")]
use rspack_cacheable::{
  ContextGuard, Result as CacheableResult, cacheable,
  with::{Custom, CustomConverter},
};
use rustc_hash::FxHasher;

use crate::InternedPath;

/// An immutable path list whose clones share their element buffer.
///
/// Construction and cache decoding do not access an intern table. An owner can
/// deduplicate equal lists in its own collection using the precomputed hash and
/// content equality. Dropping the last clone releases the data without calling
/// back into that collection.
///
/// Empty lists need no allocation. Non-empty lists use a thin arc so a retained
/// list costs one pointer, and their hash only reads the paths' cached hashes.
#[derive(Clone, Default)]
#[cfg_attr(feature = "cacheable", cacheable(with = Custom))]
pub struct SharedPathList(Option<Arc<ListStorage>>);

struct ListStorage {
  hash: u64,
  ids: Box<[InternedPath]>,
}

impl SharedPathList {
  pub fn new(items: &[InternedPath]) -> Self {
    Self::from_vec(items.to_vec())
  }

  /// Moves the elements into a shared list without cloning path handles.
  pub fn from_vec(items: Vec<InternedPath>) -> Self {
    if items.is_empty() {
      return Self::default();
    }
    let mut hasher = FxHasher::default();
    for item in &items {
      hasher.write_u64(item.precomputed_hash());
    }
    Self(Some(Arc::new(ListStorage {
      hash: hasher.finish(),
      ids: items.into_boxed_slice(),
    })))
  }

  #[inline]
  pub fn as_slice(&self) -> &[InternedPath] {
    self.0.as_ref().map_or(&[], |storage| &storage.ids)
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.as_slice().len()
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.0.is_none()
  }
}

impl Deref for SharedPathList {
  type Target = [InternedPath];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.as_slice()
  }
}

impl AsRef<[InternedPath]> for SharedPathList {
  #[inline]
  fn as_ref(&self) -> &[InternedPath] {
    self.as_slice()
  }
}

impl fmt::Debug for SharedPathList {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_list().entries(self.as_slice().iter()).finish()
  }
}

impl PartialEq for SharedPathList {
  fn eq(&self, other: &Self) -> bool {
    match (&self.0, &other.0) {
      (Some(a), Some(b)) => Arc::ptr_eq(a, b) || a.ids == b.ids,
      (None, None) => true,
      _ => false,
    }
  }
}

impl Eq for SharedPathList {}

impl Hash for SharedPathList {
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(self.0.as_ref().map_or(0, |storage| storage.hash));
  }
}

impl FromIterator<InternedPath> for SharedPathList {
  fn from_iter<T: IntoIterator<Item = InternedPath>>(iter: T) -> Self {
    Self::from_vec(iter.into_iter().collect())
  }
}

/// Persist only the paths. The owning artifact deduplicates decoded lists when
/// inserting restored factorization results, just as it does for fresh ones.
#[cfg(feature = "cacheable")]
impl CustomConverter for SharedPathList {
  type Target = Vec<InternedPath>;

  fn serialize(&self, _guard: &ContextGuard) -> CacheableResult<Self::Target> {
    Ok(self.as_slice().to_vec())
  }

  fn deserialize(data: Self::Target, _guard: &ContextGuard) -> CacheableResult<Self> {
    Ok(Self::from_vec(data))
  }
}
