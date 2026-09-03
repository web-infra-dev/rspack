use std::{
  borrow::{Borrow, Cow},
  cmp::Ordering,
  fmt,
  hash::{Hash, Hasher},
  ops::Deref,
  rc::Rc,
  sync::Arc,
};

use rspack_cacheable::{
  rkyv::{
    Place,
    rancor::{Fallible, Source},
    ser::Writer,
    string::{ArchivedString, StringResolver},
    with::{ArchiveWith, DeserializeWith, SerializeWith},
  },
  with::AsPreset,
};
use serde::{
  Deserialize, Deserializer, Serialize, Serializer,
  de::{Error as DeserializeError, Visitor},
};

/// An owned JavaScript name used by Rspack's internal IR.
///
/// The inner string is compact and interned. Equality uses string contents,
/// and hashing is compatible with `str` so maps keyed by `Atom` can be queried
/// with a borrowed string without materializing an owned value.
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

/// A hash-table key for owned-only lookups of [`Atom`] values.
///
/// Unlike [`Atom`], this type deliberately does not implement `Borrow<str>`:
/// it hashes with hstr's cached hash instead of hashing the string contents on
/// every probe. Use it only for tables queried with owned atoms or with
/// [`AtomKey::from_atom_ref`].
#[derive(Clone, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct AtomKey(Atom);

impl AtomKey {
  #[inline]
  pub fn from_atom_ref(value: &Atom) -> &Self {
    // SAFETY: `AtomKey` is transparent over `Atom` and adds no invariants.
    unsafe { &*(std::ptr::from_ref(value).cast::<Self>()) }
  }

  #[inline]
  pub fn as_atom(&self) -> &Atom {
    &self.0
  }
}

impl From<Atom> for AtomKey {
  #[inline]
  fn from(value: Atom) -> Self {
    Self(value)
  }
}

impl From<&str> for AtomKey {
  #[inline]
  fn from(value: &str) -> Self {
    Self(Atom::from(value))
  }
}

impl Hash for AtomKey {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.0.hash(state);
  }
}

impl fmt::Debug for AtomKey {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Debug::fmt(&self.0, formatter)
  }
}

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
    self.as_str().hash(state);
  }
}

impl Borrow<str> for Atom {
  #[inline]
  fn borrow(&self) -> &str {
    self.as_str()
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

impl PartialEq<&str> for Atom {
  #[inline]
  fn eq(&self, other: &&str) -> bool {
    self.as_str() == *other
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
  };
}

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

impl ArchiveWith<Atom> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = StringResolver;

  #[inline]
  fn resolve_with(field: &Atom, resolver: Self::Resolver, out: Place<Self::Archived>) {
    ArchivedString::resolve_from_str(field.as_str(), resolver, out);
  }
}

impl<S> SerializeWith<Atom, S> for AsPreset
where
  S: ?Sized + Fallible + Writer,
  S::Error: Source,
{
  #[inline]
  fn serialize_with(field: &Atom, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    ArchivedString::serialize_from_str(field.as_str(), serializer)
  }
}

impl<D> DeserializeWith<ArchivedString, Atom, D> for AsPreset
where
  D: ?Sized + Fallible,
{
  #[inline]
  fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<Atom, D::Error> {
    Ok(Atom::from(field.as_str()))
  }
}
