use std::{
  collections::{HashMap, HashSet},
  convert::From,
  fmt,
  hash::{BuildHasherDefault, Hasher},
  ops::Deref,
};

use dashmap::{DashMap, DashSet};
use hashlink::{LinkedHashMap, LinkedHashSet};
use indexmap::{IndexMap, IndexSet};
use rspack_cacheable::{
  ContextGuard, Error as CacheableError, cacheable,
  utils::PortableString,
  with::{Custom, CustomConverter},
};
use serde::Serialize;
use ustr::Ustr;

pub trait Identifiable {
  fn identifier(&self) -> Identifier;
}

/// Identity hasher for [Identifier] keys: `Ustr` already carries a precomputed
/// `FxHash`, so hashing only has to move that `u64` through.
///
/// Deliberately defined here rather than re-exported from `ustr`. The codspeed
/// profile builds with `lto = "off"`, and `ustr`'s version derives `Default`,
/// leaving `default()` without `#[inline]` and so un-inlinable across crates —
/// every `IdentifierMap` lookup then pays a real call to build a zero-sized
/// hasher.
#[derive(Debug, Clone, Copy)]
pub struct IdentifierHasher {
  hash: u64,
}

impl Default for IdentifierHasher {
  #[inline]
  fn default() -> Self {
    Self { hash: 0 }
  }
}

impl Hasher for IdentifierHasher {
  /// Only an 8-byte write carries a precomputed hash; `Ustr` writes exactly
  /// that. Anything else leaves the hash at 0, matching `ustr`'s behaviour.
  #[inline]
  fn write(&mut self, bytes: &[u8]) {
    if let Ok(bytes) = <[u8; 8]>::try_from(bytes) {
      self.hash = u64::from_ne_bytes(bytes);
    }
  }

  #[inline]
  fn finish(&self) -> u64 {
    self.hash
  }
}

/// A standard `HashMap` using `Ustr` as the key type with a custom `Hasher` that
/// just uses the precomputed hash for speed instead of calculating it
pub type IdentifierMap<V> = HashMap<Identifier, V, BuildHasherDefault<IdentifierHasher>>;
pub type IdentifierIndexMap<V> = IndexMap<Identifier, V, BuildHasherDefault<IdentifierHasher>>;
pub type IdentifierDashMap<V> = DashMap<Identifier, V, BuildHasherDefault<IdentifierHasher>>;
pub type IdentifierLinkedMap<V> =
  LinkedHashMap<Identifier, V, BuildHasherDefault<IdentifierHasher>>;

/// A standard `HashSet` using `Ustr` as the key type with a custom `Hasher` that
/// just uses the precomputed hash for speed instead of calculating it
pub type IdentifierSet = HashSet<Identifier, BuildHasherDefault<IdentifierHasher>>;
pub type IdentifierIndexSet = IndexSet<Identifier, BuildHasherDefault<IdentifierHasher>>;
pub type IdentifierDashSet = DashSet<Identifier, BuildHasherDefault<IdentifierHasher>>;
pub type IdentifierLinkedSet = LinkedHashSet<Identifier, BuildHasherDefault<IdentifierHasher>>;

#[cacheable(with=Custom, hashable)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct Identifier(Ustr);

impl Deref for Identifier {
  type Target = Ustr;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Ustr> for Identifier {
  fn from(s: Ustr) -> Self {
    Self(s)
  }
}

impl From<&str> for Identifier {
  fn from(s: &str) -> Self {
    Self(Ustr::from(s))
  }
}

impl From<String> for Identifier {
  fn from(s: String) -> Self {
    Self(Ustr::from(&s))
  }
}

impl From<Identifier> for Ustr {
  fn from(val: Identifier) -> Self {
    val.0
  }
}

impl Identifier {
  /// Convert [Identifier] to [String]
  ///
  /// Shadowed the [fmt::Display] to specialize `to_string`,
  /// like how other structs are shadowed in the standard library.
  /// See: https://github.com/rust-lang/rust/pull/32586
  ///
  /// Consistency:
  /// The result of `to_string` should be the same as the result of [fmt::Display::fmt].
  #[allow(clippy::inherent_to_string_shadow_display)]
  pub fn to_string(&self) -> String {
    self.0.to_owned()
  }

  pub fn precomputed_hash(&self) -> u64 {
    self.0.precomputed_hash()
  }
}

impl fmt::Display for Identifier {
  /// Consistency:
  /// The result of `to_string` should be the same as the result of [fmt::Display::fmt].
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

// for Identifier
impl CustomConverter for Identifier {
  type Target = PortableString;
  fn serialize(&self, guard: &ContextGuard) -> Result<Self::Target, CacheableError> {
    Ok(PortableString::new(self.as_str(), guard.project_root()))
  }
  fn deserialize(data: Self::Target, guard: &ContextGuard) -> Result<Self, CacheableError> {
    Ok(Self::from(data.into_path_string(guard.project_root())))
  }
}

#[cfg(test)]
mod tests {
  use std::hash::{BuildHasher, Hash};

  use super::*;

  /// `IdentifierMap` is only correct while hashing an [Identifier] reproduces
  /// the `FxHash` the interner already computed — that is what makes lookups a
  /// single `write_u64` instead of a walk over the string.
  #[test]
  fn hashing_yields_the_precomputed_hash() {
    let id = Identifier::from("some/module/identifier.js");
    let mut hasher = IdentifierHasher::default();
    id.hash(&mut hasher);

    assert_eq!(hasher.finish(), id.precomputed_hash());
  }

  #[test]
  fn distinct_identifiers_hash_distinctly() {
    let a = Identifier::from("a.js");
    let b = Identifier::from("b.js");
    let builder = BuildHasherDefault::<IdentifierHasher>::default();

    assert_ne!(builder.hash_one(a), builder.hash_one(b));
    assert_eq!(
      builder.hash_one(a),
      builder.hash_one(Identifier::from("a.js"))
    );
  }

  #[test]
  fn round_trips_through_an_identifier_map() {
    let mut map = IdentifierMap::default();
    let key = Identifier::from("entry.js");
    map.insert(key, 42usize);

    assert_eq!(map.get(&key), Some(&42));
    assert_eq!(map.get(&Identifier::from("entry.js")), Some(&42));
    assert_eq!(map.get(&Identifier::from("other.js")), None);
  }
}
