use std::{
  hash::BuildHasherDefault,
  sync::{Arc, LazyLock, Weak},
};

use dashmap::DashMap;
use rkyv::{
  Place,
  rancor::Fallible,
  ser::Writer,
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};
use rustc_hash::FxHasher;
use serde_json::{Map, Value};

use super::AsPreset;
use crate::{Error, Result};

pub struct SerdeJsonResolver {
  inner: StringResolver,
  value: String,
}

// for Value
impl ArchiveWith<Value> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = SerdeJsonResolver;

  #[inline]
  fn resolve_with(_field: &Value, resolver: Self::Resolver, out: Place<Self::Archived>) {
    let SerdeJsonResolver { inner, value } = resolver;
    ArchivedString::resolve_from_str(&value, inner, out);
  }
}

impl<S> SerializeWith<Value, S> for AsPreset
where
  S: Fallible<Error = Error> + Writer,
{
  #[inline]
  fn serialize_with(field: &Value, serializer: &mut S) -> Result<Self::Resolver> {
    let value = simd_json::to_string(field)
      .map_err(|_| Error::MessageError("serialize serde_json value failed"))?;
    let inner = ArchivedString::serialize_from_str(&value, serializer)?;
    Ok(SerdeJsonResolver { value, inner })
  }
}

impl<D> DeserializeWith<ArchivedString, Value, D> for AsPreset
where
  D: Fallible<Error = Error>,
{
  #[inline]
  fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<Value> {
    serde_json::from_str(field)
      .map_err(|_| Error::MessageError("deserialize serde_json value failed"))
  }
}

// for Map<String, Value>
impl ArchiveWith<Map<String, Value>> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = SerdeJsonResolver;

  #[inline]
  fn resolve_with(
    _field: &Map<String, Value>,
    resolver: Self::Resolver,
    out: Place<Self::Archived>,
  ) {
    let SerdeJsonResolver { inner, value } = resolver;
    ArchivedString::resolve_from_str(&value, inner, out);
  }
}

impl<S> SerializeWith<Map<String, Value>, S> for AsPreset
where
  S: Fallible<Error = Error> + Writer,
{
  #[inline]
  fn serialize_with(field: &Map<String, Value>, serializer: &mut S) -> Result<Self::Resolver> {
    let value = simd_json::to_string(field)
      .map_err(|_| Error::MessageError("serialize serde_json value failed"))?;
    let inner = ArchivedString::serialize_from_str(&value, serializer)?;
    Ok(SerdeJsonResolver { value, inner })
  }
}

impl<D> DeserializeWith<ArchivedString, Map<String, Value>, D> for AsPreset
where
  D: Fallible<Error = Error>,
{
  #[inline]
  fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<Map<String, Value>> {
    serde_json::from_str(field)
      .map_err(|_| Error::MessageError("deserialize serde_json value failed"))
  }
}

/// Deserialized `Arc<Value>`s keyed by their serialized JSON text.
///
/// Many cache items carry the same JSON (every module of a package stores that
/// package's `package.json`). Interning makes them share one parsed value
/// after a warm start, as they do in a fresh build, and parses each distinct
/// text once. Values are held weakly, so the map never keeps one alive; it
/// grows only with the number of distinct texts.
#[cfg_attr(allocative, rspack_allocative::root)]
static CACHED_ARC_JSON: LazyLock<DashMap<Box<str>, Weak<Value>, BuildHasherDefault<FxHasher>>> =
  LazyLock::new(Default::default);

// for Arc<Value>
impl ArchiveWith<Arc<Value>> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = SerdeJsonResolver;

  #[inline]
  fn resolve_with(field: &Arc<Value>, resolver: Self::Resolver, out: Place<Self::Archived>) {
    <Self as ArchiveWith<Value>>::resolve_with(field, resolver, out)
  }
}

impl<S> SerializeWith<Arc<Value>, S> for AsPreset
where
  S: Fallible<Error = Error> + Writer,
{
  #[inline]
  fn serialize_with(field: &Arc<Value>, serializer: &mut S) -> Result<Self::Resolver> {
    <Self as SerializeWith<Value, S>>::serialize_with(field, serializer)
  }
}

impl<D> DeserializeWith<ArchivedString, Arc<Value>, D> for AsPreset
where
  D: Fallible<Error = Error>,
{
  fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<Arc<Value>> {
    let text = field.as_str();
    if let Some(value) = CACHED_ARC_JSON.get(text).and_then(|weak| weak.upgrade()) {
      return Ok(value);
    }
    let value: Arc<Value> = Arc::new(
      serde_json::from_str(text)
        .map_err(|_| Error::MessageError("deserialize serde_json value failed"))?,
    );
    CACHED_ARC_JSON.insert(text.into(), Arc::downgrade(&value));
    Ok(value)
  }
}
