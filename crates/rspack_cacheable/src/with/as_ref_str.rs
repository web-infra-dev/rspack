use rkyv::{
  Place,
  rancor::{Fallible, Source},
  ser::Writer,
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

pub struct AsRefStr;

pub trait AsRefStrConverter {
  fn as_str(&self) -> &str;
  fn from_str(s: &str) -> Self
  where
    Self: Sized;
}

impl<T> ArchiveWith<T> for AsRefStr
where
  T: AsRefStrConverter,
{
  type Archived = ArchivedString;
  type Resolver = StringResolver;

  #[inline]
  fn resolve_with(field: &T, resolver: Self::Resolver, out: Place<Self::Archived>) {
    ArchivedString::resolve_from_str(field.as_str(), resolver, out);
  }
}

impl<T, S> SerializeWith<T, S> for AsRefStr
where
  T: AsRefStrConverter,
  S: Fallible + Writer + ?Sized,
  S::Error: Source,
{
  #[inline]
  fn serialize_with(field: &T, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    ArchivedString::serialize_from_str(field.as_str(), serializer)
  }
}

impl<T, D> DeserializeWith<ArchivedString, T, D> for AsRefStr
where
  T: AsRefStrConverter,
  D: Fallible + ?Sized,
{
  #[inline]
  fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<T, D::Error> {
    Ok(AsRefStrConverter::from_str(field.as_str()))
  }
}

// for Cow<'static, str>
impl AsRefStrConverter for std::borrow::Cow<'static, str> {
  fn as_str(&self) -> &str {
    self.as_ref()
  }
  fn from_str(s: &str) -> Self
  where
    Self: Sized,
  {
    Self::from(String::from(s))
  }
}

// for Arc<str>
use std::sync::{Arc, LazyLock};

use dashmap::DashSet;

#[cfg_attr(allocative, allocative::root)]
pub(super) static CACHED_ARC_STR: LazyLock<DashSet<Arc<str>>> = LazyLock::new(Default::default);

impl AsRefStrConverter for Arc<str> {
  fn as_str(&self) -> &str {
    self.as_ref()
  }
  fn from_str(s: &str) -> Self
  where
    Self: Sized,
  {
    if let Some(cached_str) = CACHED_ARC_STR.get(s) {
      cached_str.clone()
    } else {
      let s: Arc<str> = Arc::from(s);
      CACHED_ARC_STR.insert(s.clone());
      s
    }
  }
}

// for Box<str>
impl AsRefStrConverter for Box<str> {
  fn as_str(&self) -> &str {
    self
  }
  fn from_str(s: &str) -> Self
  where
    Self: Sized,
  {
    s.into()
  }
}

// for Ustr
impl AsRefStrConverter for ustr::Ustr {
  fn as_str(&self) -> &str {
    self
  }
  fn from_str(s: &str) -> Self
  where
    Self: Sized,
  {
    s.into()
  }
}
