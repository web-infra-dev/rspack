use indexmap::IndexMap;
use napi::{
  Either, JsValue, KeyCollectionMode, KeyConversion, KeyFilter, Unknown, ValueType,
  bindgen_prelude::{Array, Either3, FromNapiValue, JsObjectValue, Object},
};
use rspack_error::{Result, error};
use rustc_hash::FxBuildHasher;

use crate::threadsafe_function::DynThreadsafeFunction;

pub type ObjectValue = IndexMap<String, Value, FxBuildHasher>;

/// An owned snapshot of a JS value. The caller controls which properties to read
/// and how to capture values that need application-specific handling.
#[derive(Debug)]
pub enum Value {
  Null,
  Bool(bool),
  Number(f64),
  String(String),
  Array(Vec<Self>),
  Object(ObjectValue),
  Function(DynThreadsafeFunction),
  Unsupported,
}

impl Value {
  pub fn capture<'env>(
    value: Unknown<'env>,
    read_special: impl FnOnce(Unknown<'env>, ValueType) -> napi::Result<Self>,
  ) -> napi::Result<Self> {
    let kind = value.get_type()?;
    // Each cast uses the type returned by napi_typeof.
    Ok(match kind {
      ValueType::Null | ValueType::Undefined => Self::Null,
      ValueType::Boolean => Self::Bool(unsafe { value.cast()? }),
      ValueType::Number => Self::Number(unsafe { value.cast()? }),
      ValueType::String => Self::String(unsafe { value.cast()? }),
      _ => return read_special(value, kind),
    })
  }

  pub fn capture_array<'env>(
    array: Array<'env>,
    mut read: impl FnMut(Option<Unknown<'env>>) -> napi::Result<Self>,
  ) -> napi::Result<Self> {
    let mut values = Vec::with_capacity(array.len() as usize);
    for index in 0..array.len() {
      values.push(read(array.get(index)?)?);
    }
    Ok(Self::Array(values))
  }
}

pub fn capture_object<'env, K: AsRef<str> + Into<String>>(
  object: Object<'env>,
  keys: impl IntoIterator<Item = K>,
  mut read: impl FnMut(&str, Unknown<'env>) -> napi::Result<Value>,
) -> napi::Result<ObjectValue> {
  let mut fields = ObjectValue::default();
  for key in keys {
    if let Some(value) = object.get(key.as_ref())? {
      let value = read(key.as_ref(), value)?;
      fields.insert(key.into(), value);
    }
  }
  Ok(fields)
}

/// The string keys that would be visited by `Object.entries`.
pub fn enumerable_own_keys(object: &Object<'_>) -> napi::Result<Vec<String>> {
  let keys = object.get_all_property_names(
    KeyCollectionMode::OwnOnly,
    KeyFilter::Enumerable,
    KeyConversion::NumbersToStrings,
  )?;
  let keys = Array::from_unknown(keys.to_unknown())?;
  let mut names = Vec::with_capacity(keys.len() as usize);
  for index in 0..keys.len() {
    if let Some(key) = keys.get::<Unknown>(index)?
      && key.get_type()? == ValueType::String
    {
      names.push(unsafe { key.cast()? });
    }
  }
  Ok(names)
}

pub trait FromObject: Sized {
  const FIELDS: &'static [&'static str];

  fn from_object(fields: &ObjectValue) -> Result<Self>;
}

pub trait FromValue: Sized {
  fn from_value(value: &Value) -> Option<Self>;
}

impl<T: FromValue> FromValue for Option<T> {
  fn from_value(value: &Value) -> Option<Self> {
    match value {
      Value::Null => Some(None),
      value => T::from_value(value).map(Some),
    }
  }
}

impl<T: FromValue> FromValue for Vec<T> {
  fn from_value(value: &Value) -> Option<Self> {
    match value {
      Value::Array(values) => values.iter().map(T::from_value).collect(),
      _ => None,
    }
  }
}

impl<A: FromValue, B: FromValue> FromValue for Either<A, B> {
  fn from_value(value: &Value) -> Option<Self> {
    A::from_value(value)
      .map(Self::A)
      .or_else(|| B::from_value(value).map(Self::B))
  }
}

impl<A: FromValue, B: FromValue, C: FromValue> FromValue for Either3<A, B, C> {
  fn from_value(value: &Value) -> Option<Self> {
    A::from_value(value)
      .map(Self::A)
      .or_else(|| B::from_value(value).map(Self::B))
      .or_else(|| C::from_value(value).map(Self::C))
  }
}

macro_rules! from_scalar {
  ($($ty:ty => $variant:ident($value:ident) => $convert:expr),+ $(,)?) => {
    $(impl FromValue for $ty {
      fn from_value(value: &Value) -> Option<Self> {
        match value {
          Value::$variant($value) => Some($convert),
          _ => None,
        }
      }
    })+
  };
}

from_scalar! {
  bool => Bool(value) => *value,
  f64 => Number(value) => *value,
  u32 => Number(value) => value.trunc().rem_euclid(4_294_967_296.0) as u32,
  String => String(value) => value.clone(),
}

pub fn read_field<T: FromValue>(fields: &ObjectValue, key: &str) -> Result<T> {
  T::from_value(fields.get(key).unwrap_or(&Value::Null))
    .ok_or_else(|| error!("Invalid value for option '{key}'"))
}

pub fn read_optional_field_with<T>(
  fields: &ObjectValue,
  key: &str,
  convert: impl FnOnce(&Value) -> Option<T>,
) -> Result<Option<T>> {
  match fields.get(key) {
    Some(value) if !matches!(value, Value::Null) => convert(value)
      .map(Some)
      .ok_or_else(|| error!("Invalid value for option '{key}'")),
    _ => Ok(None),
  }
}
