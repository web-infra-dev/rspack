use std::sync::Arc;

use indexmap::IndexMap;
use napi::{
  Either, JsValue, KeyCollectionMode, KeyConversion, KeyFilter, Unknown, ValueType,
  bindgen_prelude::{Array, Either3, FromNapiValue, JsObjectValue, Object},
};
use rspack_error::{Result, error};
use rustc_hash::{FxBuildHasher, FxHashMap};

use crate::threadsafe_function::DynThreadsafeFunction;

pub type ObjectValue = IndexMap<String, Value, FxBuildHasher>;

/// A value in an owned configuration graph.
#[derive(Debug)]
pub enum Value {
  Null,
  Bool(bool),
  Number(f64),
  String(String),
  Array(Vec<Self>),
  Object(ObjectValue),
  Function(DynThreadsafeFunction),
  Ref(usize),
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
}

/// Object and array nodes are stored once. Edges refer to their indexes.
#[derive(Debug)]
pub struct ValueGraph {
  root: usize,
  nodes: Arc<Vec<Value>>,
}

impl ValueGraph {
  pub fn capture<'env>(
    graph: Object<'env>,
    read_function: impl Fn(Unknown<'env>) -> napi::Result<Value>,
  ) -> napi::Result<Self> {
    let root = graph
      .get::<Unknown>("root")?
      .ok_or_else(|| napi::Error::from_reason("Missing configuration graph root"))
      .and_then(read_id)?;
    let array = graph
      .get::<Array>("nodes")?
      .ok_or_else(|| napi::Error::from_reason("Missing configuration graph nodes"))?;
    let read = |value| {
      Value::capture(value, |value, kind| match kind {
        ValueType::Function => read_function(value),
        ValueType::Object if value.is_array()? => {
          let reference = Array::from_unknown(value)?;
          if reference.len() == 1
            && let Some(id) = reference.get::<Unknown>(0)?
          {
            return read_id(id).map(Value::Ref);
          }
          Err(napi::Error::from_reason(
            "Invalid configuration graph reference",
          ))
        }
        _ => Ok(Value::Unsupported),
      })
    };
    let mut nodes = Vec::with_capacity(array.len() as usize);
    for index in 0..array.len() {
      let value = array
        .get::<Unknown>(index)?
        .ok_or_else(|| napi::Error::from_reason("Missing configuration graph node"))?;
      if value.get_type()? != ValueType::Object {
        return Err(napi::Error::from_reason(
          "Expected a configuration graph node",
        ));
      }
      let node = if value.is_array()? {
        let entries = Array::from_unknown(value)?;
        let values = (0..entries.len())
          .map(|i| entries.get::<Unknown>(i)?.map_or(Ok(Value::Null), &read))
          .collect::<napi::Result<Vec<_>>>()?;
        Value::Array(values)
      } else {
        let object = value.coerce_to_object()?;
        let keys = enumerable_own_keys(&object)?;
        let mut fields = ObjectValue::with_capacity_and_hasher(keys.len(), FxBuildHasher);
        for key in keys {
          if let Some(value) = object.get::<Unknown>(&key)? {
            fields.insert(key, read(value)?);
          }
        }
        Value::Object(fields)
      };
      nodes.push(node);
    }
    validate_graph(root, &nodes)?;
    Ok(Self {
      root,
      nodes: Arc::new(nodes),
    })
  }

  pub fn root(&self) -> ValueRef<'_> {
    ValueRef {
      value: &self.nodes[self.root],
      nodes: &self.nodes,
    }
  }
}

fn read_id(value: Unknown<'_>) -> napi::Result<usize> {
  if value.get_type()? == ValueType::Number {
    let id: f64 = unsafe { value.cast()? };
    if id >= 0.0 && id <= f64::from(u32::MAX) && id.fract() == 0.0 {
      return Ok(id as usize);
    }
  }
  Err(napi::Error::from_reason(
    "Invalid configuration graph node ID",
  ))
}

fn references(node: &Value) -> impl Iterator<Item = usize> + '_ {
  let array = match node {
    Value::Array(values) => Some(values),
    _ => None,
  };
  let object = match node {
    Value::Object(fields) => Some(fields),
    _ => None,
  };
  array
    .into_iter()
    .flatten()
    .chain(object.into_iter().flat_map(|fields| fields.values()))
    .filter_map(|value| match value {
      Value::Ref(id) => Some(*id),
      _ => None,
    })
}

// Typed configuration is a tree. Detect back-edges once before decoding, while
// allowing the same completed node to be used from multiple branches.
fn validate_graph(root: usize, nodes: &[Value]) -> napi::Result<()> {
  #[derive(Clone, Copy)]
  enum Visit {
    New,
    Active,
    Done,
  }
  let invalid_ref = || napi::Error::from_reason("Configuration graph reference is out of bounds");
  let mut visits = vec![Visit::New; nodes.len()];
  let mut stack = vec![(root, references(nodes.get(root).ok_or_else(invalid_ref)?))];
  visits[root] = Visit::Active;
  while let Some((id, children)) = stack.last_mut() {
    if let Some(child) = children.next() {
      match visits.get(child).ok_or_else(invalid_ref)? {
        Visit::New => {
          visits[child] = Visit::Active;
          stack.push((child, references(&nodes[child])));
        }
        Visit::Active => {
          return Err(napi::Error::from_reason(
            "Circular reference in configuration object",
          ));
        }
        Visit::Done => {}
      }
    } else {
      visits[*id] = Visit::Done;
      stack.pop();
    }
  }
  Ok(())
}

#[derive(Clone, Copy)]
pub struct ValueRef<'a> {
  value: &'a Value,
  nodes: &'a Arc<Vec<Value>>,
}

impl<'a> ValueRef<'a> {
  pub fn value(self) -> &'a Value {
    match self.value {
      Value::Ref(id) => &self.nodes[*id],
      value => value,
    }
  }

  pub fn child(self, value: &'a Value) -> Self {
    Self { value, ..self }
  }

  pub fn subgraph(self) -> Option<ValueGraph> {
    let Value::Ref(root) = self.value else {
      return None;
    };
    Some(ValueGraph {
      root: *root,
      nodes: Arc::clone(self.nodes),
    })
  }

  pub fn object(self) -> Option<ObjectRef<'a>> {
    match self.value() {
      Value::Object(fields) => Some(ObjectRef {
        fields,
        nodes: self.nodes,
      }),
      _ => None,
    }
  }
}

#[derive(Clone, Copy)]
pub struct ObjectRef<'a> {
  fields: &'a ObjectValue,
  nodes: &'a Arc<Vec<Value>>,
}

impl<'a> ObjectRef<'a> {
  pub fn get(self, key: &str) -> Option<ValueRef<'a>> {
    self.fields.get(key).map(|value| ValueRef {
      value,
      nodes: self.nodes,
    })
  }

  pub fn iter(self) -> impl Iterator<Item = (&'a String, ValueRef<'a>)> {
    self.fields.iter().map(move |(key, value)| {
      (
        key,
        ValueRef {
          value,
          nodes: self.nodes,
        },
      )
    })
  }
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

  fn from_object(fields: ObjectRef<'_>) -> Result<Self>;
}

pub trait FromValue: Sized {
  fn from_value(value: ValueRef<'_>) -> Option<Self>;
}

impl<T: FromValue> FromValue for Option<T> {
  fn from_value(value: ValueRef<'_>) -> Option<Self> {
    match value.value() {
      Value::Null => Some(None),
      _ => T::from_value(value).map(Some),
    }
  }
}

impl<T: FromValue> FromValue for Vec<T> {
  fn from_value(value: ValueRef<'_>) -> Option<Self> {
    match value.value() {
      Value::Array(values) => values
        .iter()
        .map(|item| T::from_value(value.child(item)))
        .collect(),
      _ => None,
    }
  }
}

impl<T: FromValue> FromValue for FxHashMap<String, T> {
  fn from_value(value: ValueRef<'_>) -> Option<Self> {
    value
      .object()?
      .iter()
      .map(|(key, value)| Some((key.clone(), T::from_value(value)?)))
      .collect()
  }
}

impl<A: FromValue, B: FromValue> FromValue for Either<A, B> {
  fn from_value(value: ValueRef<'_>) -> Option<Self> {
    A::from_value(value)
      .map(Self::A)
      .or_else(|| B::from_value(value).map(Self::B))
  }
}

impl<A: FromValue, B: FromValue, C: FromValue> FromValue for Either3<A, B, C> {
  fn from_value(value: ValueRef<'_>) -> Option<Self> {
    A::from_value(value)
      .map(Self::A)
      .or_else(|| B::from_value(value).map(Self::B))
      .or_else(|| C::from_value(value).map(Self::C))
  }
}

macro_rules! from_scalar {
  ($($ty:ty => $variant:ident($value:ident) => $convert:expr),+ $(,)?) => {
    $(impl FromValue for $ty {
      fn from_value(value: ValueRef<'_>) -> Option<Self> {
        match value.value() {
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

pub fn read_field<T: FromValue>(fields: ObjectRef<'_>, key: &str) -> Result<T> {
  let value = fields.get(key).unwrap_or(ValueRef {
    value: &Value::Null,
    nodes: fields.nodes,
  });
  T::from_value(value).ok_or_else(|| error!("Invalid value for option '{key}'"))
}

pub fn read_optional_field_with<T>(
  fields: ObjectRef<'_>,
  key: &str,
  convert: impl FnOnce(ValueRef<'_>) -> Option<T>,
) -> Result<Option<T>> {
  match fields.get(key) {
    Some(value) if !matches!(value.value(), Value::Null) => convert(value)
      .map(Some)
      .ok_or_else(|| error!("Invalid value for option '{key}'")),
    _ => Ok(None),
  }
}
