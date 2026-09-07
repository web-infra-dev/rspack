use std::{
  borrow::Cow,
  cmp::Ordering,
  collections::hash_map::Entry as HashEntry,
  fmt,
  hash::{BuildHasherDefault, Hash, Hasher},
  ops::{Deref, DerefMut},
  rc::Rc,
  sync::Arc,
};

use indexmap::{IndexMap, IndexSet};
use ref_cast::RefCast;
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};
use serde::{
  Deserialize, Deserializer, Serialize, Serializer,
  de::{Error as DeserializeError, Visitor},
};
#[cfg(feature = "swc")]
use swc_core::atoms::Atom as SwcAtom;

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;
type FxIndexSet<K> = IndexSet<K, BuildHasherDefault<FxHasher>>;

/// An owned JavaScript name used by Rspack's internal IR.
///
/// The inner string is compact and interned. Equality uses string contents and
/// hashing reuses hstr's cached hash.
#[derive(Clone, Default)]
#[repr(transparent)]
pub struct Atom(hstr::Atom);

impl Atom {
  #[inline]
  pub fn new<S>(value: S) -> Self
  where
    hstr::Atom: From<S>,
  {
    Self(hstr::Atom::from(value))
  }

  #[inline]
  pub fn as_str(&self) -> &str {
    self.0.as_ref()
  }
}

/// An owned or borrowed atom lookup. Collection users do not need to select a
/// hashing strategy; [`AtomMap`] and [`AtomSet`] dispatch internally.
#[derive(Clone, Copy)]
pub enum AtomRef<'a> {
  Atom(&'a Atom),
  Str(&'a str),
}

impl PartialEq for AtomRef<'_> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.as_str() == other.as_str()
  }
}

impl Eq for AtomRef<'_> {}

impl PartialEq<str> for AtomRef<'_> {
  #[inline]
  fn eq(&self, other: &str) -> bool {
    self.as_str() == other
  }
}

impl PartialEq<&str> for AtomRef<'_> {
  #[inline]
  fn eq(&self, other: &&str) -> bool {
    self.as_str() == *other
  }
}

impl PartialEq<Atom> for AtomRef<'_> {
  #[inline]
  fn eq(&self, other: &Atom) -> bool {
    self.as_str() == other.as_str()
  }
}

impl PartialEq<AtomRef<'_>> for Atom {
  #[inline]
  fn eq(&self, other: &AtomRef<'_>) -> bool {
    self.as_str() == other.as_str()
  }
}

impl PartialEq<AtomRef<'_>> for &Atom {
  #[inline]
  fn eq(&self, other: &AtomRef<'_>) -> bool {
    self.as_str() == other.as_str()
  }
}

impl<'a> AtomRef<'a> {
  #[inline]
  pub fn as_str(self) -> &'a str {
    match self {
      Self::Atom(atom) => atom.as_str(),
      Self::Str(value) => value,
    }
  }

  #[inline]
  pub fn to_atom(self) -> Atom {
    match self {
      Self::Atom(atom) => atom.clone(),
      Self::Str(value) => Atom::from(value),
    }
  }
}

impl<'a> From<&'a Atom> for AtomRef<'a> {
  #[inline]
  fn from(value: &'a Atom) -> Self {
    Self::Atom(value)
  }
}

impl<'a> From<&'a str> for AtomRef<'a> {
  #[inline]
  fn from(value: &'a str) -> Self {
    Self::Str(value)
  }
}

impl<'a> From<&'a String> for AtomRef<'a> {
  #[inline]
  fn from(value: &'a String) -> Self {
    Self::Str(value)
  }
}

#[cfg(feature = "swc")]
impl<'a> From<&'a SwcAtom> for AtomRef<'a> {
  #[inline]
  fn from(value: &'a SwcAtom) -> Self {
    Self::Str(value.as_str())
  }
}

#[inline]
fn hash_str_as_atom<H: Hasher>(value: &str, state: &mut H) {
  // hstr stores short atoms inline and uses their tagged representation as
  // the cached hash. `inline_atom` reproduces that value without allocating.
  if let Some(atom) = hstr::inline_atom(value) {
    atom.hash(state);
    return;
  }

  // Dynamic hstr atoms cache `FxHasher` over the byte slice. Reproduce that
  // prehash here, then feed it to the caller exactly as `hstr::Atom::hash`
  // does. This avoids constructing or interning an atom for string lookups.
  let mut hasher = FxHasher::default();
  value.as_bytes().hash(&mut hasher);
  state.write_u64(hasher.finish());
}

#[derive(RefCast)]
#[repr(transparent)]
struct AtomQueryStr(str);

impl PartialEq for AtomQueryStr {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl Eq for AtomQueryStr {}

impl Hash for AtomQueryStr {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    hash_str_as_atom(&self.0, state);
  }
}

impl std::borrow::Borrow<AtomQueryStr> for Atom {
  #[inline]
  fn borrow(&self) -> &AtomQueryStr {
    AtomQueryStr::ref_cast(self.as_str())
  }
}

macro_rules! atom_query {
  ($key:expr, $query:ident, $operation:expr) => {{
    match $key.into() {
      AtomRef::Atom($query) => $operation,
      AtomRef::Str(value) => {
        let $query = AtomQueryStr::ref_cast(value);
        $operation
      }
    }
  }};
}

macro_rules! impl_atom_map {
  ($name:ident<$value:ident>, $inner:ty, $entry:ty; $($remove:ident -> $result:ty),+ $(,)?) => {
    impl<$value> $name<$value> {
      #[inline]
      pub fn with_capacity(capacity: usize) -> Self {
        Self {
          inner: <$inner>::with_capacity_and_hasher(capacity, Default::default()),
        }
      }

      #[inline]
      pub fn get<'key>(&self, key: impl Into<AtomRef<'key>>) -> Option<&$value> {
        atom_query!(key, query, self.inner.get(query))
      }

      #[inline]
      pub fn get_mut<'key>(&mut self, key: impl Into<AtomRef<'key>>) -> Option<&mut $value> {
        atom_query!(key, query, self.inner.get_mut(query))
      }

      #[inline]
      pub fn contains_key<'key>(&self, key: impl Into<AtomRef<'key>>) -> bool {
        atom_query!(key, query, self.inner.contains_key(query))
      }

      #[inline]
      pub fn get_key_value<'key>(
        &self,
        key: impl Into<AtomRef<'key>>,
      ) -> Option<(&Atom, &$value)> {
        atom_query!(key, query, self.inner.get_key_value(query))
      }

      #[inline]
      pub fn insert(&mut self, key: Atom, value: $value) -> Option<$value> {
        self.inner.insert(key, value)
      }

      #[inline]
      pub fn entry(&mut self, key: Atom) -> $entry {
        self.inner.entry(key)
      }

      $(
        #[inline]
        pub fn $remove<'key>(&mut self, key: impl Into<AtomRef<'key>>) -> $result {
          atom_query!(key, query, self.inner.$remove(query))
        }
      )+
    }

    impl<$value> Deref for $name<$value> {
      type Target = $inner;

      #[inline]
      fn deref(&self) -> &Self::Target {
        &self.inner
      }
    }

    impl<$value> DerefMut for $name<$value> {
      #[inline]
      fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
      }
    }

    impl<$value> From<$inner> for $name<$value> {
      #[inline]
      fn from(inner: $inner) -> Self {
        Self { inner }
      }
    }

    impl<$value> FromIterator<(Atom, $value)> for $name<$value> {
      #[inline]
      fn from_iter<T: IntoIterator<Item = (Atom, $value)>>(iter: T) -> Self {
        Self {
          inner: iter.into_iter().collect(),
        }
      }
    }

    impl<$value> Extend<(Atom, $value)> for $name<$value> {
      #[inline]
      fn extend<T: IntoIterator<Item = (Atom, $value)>>(&mut self, iter: T) {
        self.inner.extend(iter);
      }
    }

    impl<$value> IntoIterator for $name<$value> {
      type Item = (Atom, $value);
      type IntoIter = <$inner as IntoIterator>::IntoIter;

      #[inline]
      fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
      }
    }

    impl<'a, $value> IntoIterator for &'a $name<$value> {
      type Item = (&'a Atom, &'a $value);
      type IntoIter = <&'a $inner as IntoIterator>::IntoIter;

      #[inline]
      fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
      }
    }
  };
}

macro_rules! impl_atom_set {
  ($name:ident, $inner:ty; $($remove:ident -> $result:ty),+ $(,)?) => {
    impl $name {
      #[inline]
      pub fn with_capacity(capacity: usize) -> Self {
        Self {
          inner: <$inner>::with_capacity_and_hasher(capacity, Default::default()),
        }
      }

      #[inline]
      pub fn contains<'key>(&self, key: impl Into<AtomRef<'key>>) -> bool {
        atom_query!(key, query, self.inner.contains(query))
      }

      #[inline]
      pub fn get<'key>(&self, key: impl Into<AtomRef<'key>>) -> Option<&Atom> {
        atom_query!(key, query, self.inner.get(query))
      }

      #[inline]
      pub fn insert(&mut self, value: Atom) -> bool {
        self.inner.insert(value)
      }

      $(
        #[inline]
        pub fn $remove<'key>(&mut self, key: impl Into<AtomRef<'key>>) -> $result {
          atom_query!(key, query, self.inner.$remove(query))
        }
      )+
    }

    impl Deref for $name {
      type Target = $inner;

      #[inline]
      fn deref(&self) -> &Self::Target {
        &self.inner
      }
    }

    impl DerefMut for $name {
      #[inline]
      fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
      }
    }

    impl From<$inner> for $name {
      #[inline]
      fn from(inner: $inner) -> Self {
        Self { inner }
      }
    }

    impl FromIterator<Atom> for $name {
      #[inline]
      fn from_iter<T: IntoIterator<Item = Atom>>(iter: T) -> Self {
        Self {
          inner: iter.into_iter().collect(),
        }
      }
    }

    impl Extend<Atom> for $name {
      #[inline]
      fn extend<T: IntoIterator<Item = Atom>>(&mut self, iter: T) {
        self.inner.extend(iter);
      }
    }

    impl IntoIterator for $name {
      type Item = Atom;
      type IntoIter = <$inner as IntoIterator>::IntoIter;

      #[inline]
      fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
      }
    }

    impl<'a> IntoIterator for &'a $name {
      type Item = &'a Atom;
      type IntoIter = <&'a $inner as IntoIterator>::IntoIter;

      #[inline]
      fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
      }
    }
  };
}

#[derive(Clone, Debug, Default)]
pub struct AtomMap<V> {
  inner: FxHashMap<Atom, V>,
}

impl_atom_map!(
  AtomMap<V>,
  FxHashMap<Atom, V>,
  HashEntry<'_, Atom, V>;
  remove -> Option<V>,
  remove_entry -> Option<(Atom, V)>,
);

#[derive(Clone, Debug, Default)]
pub struct AtomSet {
  inner: FxHashSet<Atom>,
}

impl_atom_set!(
  AtomSet,
  FxHashSet<Atom>;
  remove -> bool,
  take -> Option<Atom>,
);

/// An insertion-ordered map keyed by [`Atom`] with zero-allocation borrowed
/// lookups. Query hashing is selected internally without changing map order.
#[derive(Clone, Debug, Default)]
pub struct IndexAtomMap<V> {
  inner: FxIndexMap<Atom, V>,
}

impl_atom_map!(
  IndexAtomMap<V>,
  FxIndexMap<Atom, V>,
  indexmap::map::Entry<'_, Atom, V>;
  shift_remove -> Option<V>,
  swap_remove -> Option<V>,
);

/// An insertion-ordered set of [`Atom`] values with zero-allocation borrowed
/// lookups. Query hashing is selected internally without changing set order.
#[derive(Clone, Debug, Default)]
pub struct IndexAtomSet {
  inner: FxIndexSet<Atom>,
}

impl_atom_set!(
  IndexAtomSet,
  FxIndexSet<Atom>;
  shift_remove -> bool,
  swap_remove -> bool,
);

impl PartialEq for Atom {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl Eq for Atom {}

impl Hash for Atom {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.hash(state);
  }
}

impl Deref for Atom {
  type Target = str;

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.as_str()
  }
}

impl AsRef<str> for Atom {
  #[inline]
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl PartialEq<str> for Atom {
  #[inline]
  fn eq(&self, other: &str) -> bool {
    self.as_str() == other
  }
}

impl PartialEq<Atom> for str {
  #[inline]
  fn eq(&self, other: &Atom) -> bool {
    self == other.as_str()
  }
}

#[cfg(feature = "swc")]
impl PartialEq<SwcAtom> for Atom {
  #[inline]
  fn eq(&self, other: &SwcAtom) -> bool {
    self.as_str() == other.as_str()
  }
}

#[cfg(feature = "swc")]
impl PartialEq<Atom> for SwcAtom {
  #[inline]
  fn eq(&self, other: &Atom) -> bool {
    self.as_str() == other.as_str()
  }
}

macro_rules! impl_partial_eq_as_ref_str {
  ($ty:ty) => {
    impl PartialEq<$ty> for Atom {
      #[inline]
      fn eq(&self, other: &$ty) -> bool {
        self.as_str() == <_ as AsRef<str>>::as_ref(other)
      }
    }

    impl PartialEq<Atom> for $ty {
      #[inline]
      fn eq(&self, other: &Atom) -> bool {
        <_ as AsRef<str>>::as_ref(self) == other.as_str()
      }
    }
  };
}

impl_partial_eq_as_ref_str!(&str);
impl_partial_eq_as_ref_str!(Box<str>);
impl_partial_eq_as_ref_str!(Arc<str>);
impl_partial_eq_as_ref_str!(Rc<str>);
impl_partial_eq_as_ref_str!(Cow<'_, str>);
impl_partial_eq_as_ref_str!(String);

impl PartialOrd for Atom {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Atom {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_str().cmp(other.as_str())
  }
}

impl fmt::Debug for Atom {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Debug::fmt(self.as_str(), formatter)
  }
}

impl fmt::Display for Atom {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Display::fmt(self.as_str(), formatter)
  }
}

macro_rules! impl_from {
  ($ty:ty) => {
    impl From<$ty> for Atom {
      #[inline]
      fn from(value: $ty) -> Self {
        Self(hstr::Atom::from(value))
      }
    }
  };
}

impl_from!(&'_ str);
impl_from!(Box<str>);
impl_from!(Cow<'_, str>);
impl_from!(String);

impl From<&String> for Atom {
  #[inline]
  fn from(value: &String) -> Self {
    Self::from(value.as_str())
  }
}

#[cfg(feature = "swc")]
impl From<SwcAtom> for Atom {
  #[inline]
  fn from(value: SwcAtom) -> Self {
    Self::from(value.as_str())
  }
}

#[cfg(feature = "swc")]
impl From<&SwcAtom> for Atom {
  #[inline]
  fn from(value: &SwcAtom) -> Self {
    Self::from(value.as_str())
  }
}

impl From<hstr::Atom> for Atom {
  #[inline]
  fn from(value: hstr::Atom) -> Self {
    Self(value)
  }
}

impl From<Atom> for hstr::Atom {
  #[inline]
  fn from(value: Atom) -> Self {
    value.0
  }
}

impl Serialize for Atom {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.as_str())
  }
}

impl<'de> Deserialize<'de> for Atom {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct AtomVisitor;

    impl<'de> Visitor<'de> for AtomVisitor {
      type Value = Atom;

      fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a string")
      }

      #[inline]
      fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
      where
        E: DeserializeError,
      {
        Ok(Atom::from(value))
      }

      #[inline]
      fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
      where
        E: DeserializeError,
      {
        Ok(Atom::from(value))
      }

      #[inline]
      fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
      where
        E: DeserializeError,
      {
        Ok(Atom::from(value))
      }
    }

    deserializer.deserialize_str(AtomVisitor)
  }
}
