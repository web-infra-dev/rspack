use std::path::Path;

use json::JsonValue;
use rspack_cacheable::{
  __private::rkyv::{
    Archive, Archived, Deserialize, Place, Resolver, Serialize,
    rancor::Fallible,
    ser::Sharing,
    with::{ArchiveWith, DeserializeWith, SerializeWith},
  },
  CacheableContext, ContextGuard, Error, Result, cacheable,
};
use rspack_error::error;
use rspack_paths::Utf8PathBuf;

#[cacheable]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JsonArchivePolicy {
  Preserve,
  DeriveFromModuleSource,
}

#[derive(Debug, Clone)]
pub(crate) struct JsonArchiveContext {
  project_path: Option<Utf8PathBuf>,
  policy: JsonArchivePolicy,
}

impl JsonArchiveContext {
  pub(crate) fn new(project_path: Option<Utf8PathBuf>, policy: JsonArchivePolicy) -> Self {
    Self {
      project_path,
      policy,
    }
  }
}

impl CacheableContext for JsonArchiveContext {
  fn project_root(&self) -> Option<&Path> {
    self.project_path.as_ref().map(|path| path.as_std_path())
  }
}

#[doc(hidden)]
#[cacheable]
#[derive(Debug)]
pub enum StoredJsonData {
  Canonical(String),
  FromModuleSource,
}

#[doc(hidden)]
pub struct JsonDataArchive;

#[doc(hidden)]
pub struct JsonDataResolver {
  value: Option<StoredJsonData>,
  resolver: Resolver<Option<StoredJsonData>>,
}

impl ArchiveWith<Option<JsonValue>> for JsonDataArchive {
  type Archived = Archived<Option<StoredJsonData>>;
  type Resolver = JsonDataResolver;

  fn resolve_with(
    _field: &Option<JsonValue>,
    resolver: Self::Resolver,
    out: Place<Self::Archived>,
  ) {
    resolver.value.resolve(resolver.resolver, out);
  }
}

impl<S> SerializeWith<Option<JsonValue>, S> for JsonDataArchive
where
  S: Fallible<Error = Error> + Sharing,
  Option<StoredJsonData>: Serialize<S>,
{
  fn serialize_with(field: &Option<JsonValue>, serializer: &mut S) -> Result<Self::Resolver> {
    let Some(value) = field else {
      let value = None;
      return Ok(JsonDataResolver {
        resolver: value.serialize(serializer)?,
        value,
      });
    };

    let policy = ContextGuard::sharing_guard(serializer)?
      .downcast_context::<JsonArchiveContext>()
      .map_or(JsonArchivePolicy::Preserve, |context| context.policy);
    let value = Some(match policy {
      JsonArchivePolicy::Preserve => StoredJsonData::Canonical(value.dump()),
      JsonArchivePolicy::DeriveFromModuleSource => StoredJsonData::FromModuleSource,
    });
    Ok(JsonDataResolver {
      resolver: value.serialize(serializer)?,
      value,
    })
  }
}

impl<D> DeserializeWith<Archived<Option<StoredJsonData>>, Option<JsonValue>, D> for JsonDataArchive
where
  D: Fallible<Error = Error>,
  Archived<Option<StoredJsonData>>: Deserialize<Option<StoredJsonData>, D>,
{
  fn deserialize_with(
    field: &Archived<Option<StoredJsonData>>,
    deserializer: &mut D,
  ) -> Result<Option<JsonValue>> {
    let value = field.deserialize(deserializer)?;
    match value {
      Some(StoredJsonData::Canonical(value)) => json::parse(&value)
        .map(Some)
        .map_err(|_| Error::MessageError("deserialize json value failed")),
      Some(StoredJsonData::FromModuleSource) | None => Ok(None),
    }
  }
}

pub(crate) fn restore_json_data_from_source(source: &str) -> rspack_error::Result<JsonValue> {
  let source = source.strip_prefix('\u{feff}').unwrap_or(source);
  json::parse(source).map_err(|cause| error!("cannot recover persisted JSON module: {cause}"))
}

#[cfg(test)]
mod tests {
  use super::restore_json_data_from_source;

  #[test]
  fn restores_source_with_exactly_one_utf8_bom() {
    let restored = restore_json_data_from_source("\u{feff}{\"value\":7}").unwrap();
    assert_eq!(restored["value"], 7);
    assert!(restore_json_data_from_source("\u{feff}\u{feff}{\"value\":7}").is_err());
  }

  #[test]
  fn restores_the_post_loader_json_source() {
    let loader_output = "{\"generated\":true,\"value\":11}";
    let restored = restore_json_data_from_source(loader_output).unwrap();
    assert_eq!(restored["generated"], true);
    assert_eq!(restored["value"], 11);
  }

  #[test]
  fn preserves_json_primitives_and_escaped_keys() {
    for source in [
      "null",
      "true",
      "17",
      "[1,2,3]",
      "{\"__proto__\":{\"safe\":true},\"\\u006bey\":\"value\"}",
    ] {
      let expected = json::parse(source).unwrap();
      assert_eq!(restore_json_data_from_source(source).unwrap(), expected);
    }
  }

  #[test]
  fn rejects_malformed_recovered_source() {
    assert!(restore_json_data_from_source("{\"value\":").is_err());
    assert!(restore_json_data_from_source("").is_err());
  }
}
