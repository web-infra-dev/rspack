use std::{
  collections::{HashMap, HashSet},
  convert::From,
  fmt,
  hash::BuildHasherDefault,
  ops::Deref,
};

use dashmap::{DashMap, DashSet};
use hashlink::{LinkedHashMap, LinkedHashSet};
use indexmap::{IndexMap, IndexSet};
use rspack_cacheable::{cacheable, with, with::AsPreset};
use serde::Serialize;
use ustr::Ustr;

pub trait Identifiable {
  fn identifier(&self) -> Identifier;
}

pub type IdentifierHasher = ustr::IdentityHasher;

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

#[cacheable(hashable)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct Identifier(#[cacheable(with=AsPreset)] Ustr);

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
impl with::AsRefStrConverter for Identifier {
  fn as_str(&self) -> &str {
    self.0.as_str()
  }
  fn from_str(s: &str) -> Self
  where
    Self: Sized,
  {
    s.into()
  }
}
