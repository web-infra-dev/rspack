use std::collections::{HashMap, HashSet};
use std::hash::BuildHasherDefault;
use std::{convert::From, fmt, ops::Deref};

use hashlink::{LinkedHashMap, LinkedHashSet};
use serde::Serialize;
use ustr::Ustr;

pub trait Identifiable {
  fn identifier(&self) -> Identifier;
}

pub type IdentifierHasher = ustr::IdentityHasher;

/// A standard `HashMap` using `Ustr` as the key type with a custom `Hasher` that
/// just uses the precomputed hash for speed instead of calculating it
pub type IdentifierMap<V> = HashMap<Identifier, V, BuildHasherDefault<IdentifierHasher>>;
pub type IdentifierLinkedMap<V> =
  LinkedHashMap<Identifier, V, BuildHasherDefault<IdentifierHasher>>;

/// A standard `HashSet` using `Ustr` as the key type with a custom `Hasher` that
/// just uses the precomputed hash for speed instead of calculating it
pub type IdentifierSet = HashSet<Identifier, BuildHasherDefault<IdentifierHasher>>;
pub type IdentifierLinkedSet = LinkedHashSet<Identifier, BuildHasherDefault<IdentifierHasher>>;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
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

impl fmt::Display for Identifier {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0.as_str())
  }
}
