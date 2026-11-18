use std::sync::{Arc, OnceLock};

use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

use super::JSONMap;

#[derive(Debug)]
struct Field {
  json: Box<str>,
  value: OnceLock<serde_json::Value>,
}

impl Field {
  fn parse(&self) -> serde_json::Value {
    // Use the same parser as the resolver, including its number handling.
    simd_json::serde::from_slice(&mut self.json.as_bytes().to_vec())
      .expect("serialized package.json field should be valid JSON")
  }

  fn get(&self) -> &serde_json::Value {
    self.value.get_or_init(|| self.parse())
  }
}

/// Shared package metadata independent of the resolver's input buffer.
///
/// Only retained fields are copied, as compact JSON. Reading a field materializes
/// its subtree once; unrelated fields remain serialized. Keeping this owner on a
/// module does not keep skipped fields or the resolver's parsed DOM alive after
/// the resolver cache is cleared.
#[derive(Debug)]
pub struct PackageJsonData {
  fields: IndexMap<Box<str>, Field, FxBuildHasher>,
  json: OnceLock<Arc<serde_json::Value>>,
}

impl PackageJsonData {
  pub(super) fn from_object(object: &JSONMap<'_>) -> Self {
    Self {
      fields: object
        .iter()
        .map(|(key, value)| {
          (
            key.as_ref().into(),
            Field {
              json: simd_json::to_string(value)
                .expect("parsed package.json field should serialize")
                .into_boxed_str(),
              value: OnceLock::new(),
            },
          )
        })
        .collect(),
      json: OnceLock::new(),
    }
  }

  pub(super) fn from_json(json: Arc<serde_json::Value>) -> Self {
    Self {
      fields: IndexMap::default(),
      json: OnceLock::from(json),
    }
  }

  /// Returns the complete JSON object, materializing it only when requested.
  pub fn json(&self) -> &Arc<serde_json::Value> {
    self.json.get_or_init(|| {
      let object = self
        .fields
        .iter()
        .map(|(key, field)| {
          // Do not populate every field cache when the whole object is requested.
          // Already borrowed field values must remain stable, so those are copied.
          let value = field.value.get().cloned().unwrap_or_else(|| field.parse());
          (key.to_string(), value)
        })
        .collect();
      Arc::new(serde_json::Value::Object(object))
    })
  }

  /// Reads an object property path. An empty path returns the complete JSON value.
  pub fn get<'a>(&self, path: impl IntoIterator<Item = &'a str>) -> Option<&serde_json::Value> {
    let mut path = path.into_iter();
    if let Some(json) = self.json.get() {
      return path.try_fold(json.as_ref(), |value, key| value.get(key));
    }
    let Some(first) = path.next() else {
      return Some(self.json());
    };
    let value = self.fields.get(first)?.get();
    path.try_fold(value, |value, key| value.get(key))
  }
}
