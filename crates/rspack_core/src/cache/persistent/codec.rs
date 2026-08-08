use rspack_cacheable::{
  __private::rkyv::{Archive, Deserialize, Serialize, bytecheck::CheckBytes},
  Deserializer, Serializer, Validator, from_bytes, to_bytes,
};
use rspack_error::Result;
use rspack_paths::Utf8PathBuf;

use crate::json_archive::{JsonArchiveContext, JsonArchivePolicy};

/// Cache codec for encoding and decoding cacheable data
///
/// This struct encapsulates the serialization and deserialization logic,
/// automatically passing the project context to rspack_cacheable's to_bytes and from_bytes.
///
/// # Example
///
/// ```ignore
/// let codec = CacheCodec::new(project_path);
///
/// // Encode data to bytes
/// let bytes = codec.encode(&my_data)?;
///
/// // Decode bytes back to data
/// let my_data: MyType = codec.decode(&bytes)?;
/// ```
#[derive(Debug, Clone)]
pub struct CacheCodec {
  context: JsonArchiveContext,
  source_json_context: JsonArchiveContext,
}

impl CacheCodec {
  pub fn new(project_path: Option<Utf8PathBuf>) -> Self {
    Self {
      context: JsonArchiveContext::new(project_path.clone(), JsonArchivePolicy::Preserve),
      source_json_context: JsonArchiveContext::new(
        project_path,
        JsonArchivePolicy::DeriveFromModuleSource,
      ),
    }
  }

  pub fn encode<T>(&self, data: &T) -> Result<Vec<u8>>
  where
    T: for<'a> Serialize<Serializer<'a>>,
  {
    to_bytes(data, &self.context).map_err(|e| rspack_error::error!(e.to_string()))
  }

  pub(crate) fn encode_with_json_policy<T>(
    &self,
    data: &T,
    json_archive_policy: JsonArchivePolicy,
  ) -> Result<Vec<u8>>
  where
    T: for<'a> Serialize<Serializer<'a>>,
  {
    let context = match json_archive_policy {
      JsonArchivePolicy::Preserve => &self.context,
      JsonArchivePolicy::DeriveFromModuleSource => &self.source_json_context,
    };
    to_bytes(data, context).map_err(|error| rspack_error::error!(error.to_string()))
  }

  pub fn decode<T>(&self, bytes: &[u8]) -> Result<T>
  where
    T: Archive,
    T::Archived: for<'a> CheckBytes<Validator<'a>> + Deserialize<T, Deserializer>,
  {
    from_bytes(bytes, &self.context).map_err(|e| rspack_error::error!(e.to_string()))
  }
}

#[cfg(test)]
mod tests {
  use json::JsonValue;
  use rspack_cacheable::{cacheable, from_bytes, to_bytes};

  use super::CacheCodec;
  use crate::json_archive::{JsonArchivePolicy, JsonDataArchive};

  #[cacheable]
  #[derive(Debug)]
  struct JsonOwner {
    #[cacheable(with=JsonDataArchive)]
    value: Option<JsonValue>,
  }

  #[test]
  fn source_policy_stores_an_explicit_compact_recoverability_marker() {
    let value = json::object! { payload: "x".repeat(16 * 1024) };
    let owner = JsonOwner { value: Some(value) };
    let codec = CacheCodec::new(None);
    let preserved = codec
      .encode_with_json_policy(&owner, JsonArchivePolicy::Preserve)
      .expect("preserved JSON should archive");
    let source_backed = codec
      .encode_with_json_policy(&owner, JsonArchivePolicy::DeriveFromModuleSource)
      .expect("source-backed JSON should archive");
    eprintln!(
      "synthetic JSON archive: preserved={} bytes, source-backed={} bytes",
      preserved.len(),
      source_backed.len()
    );
    assert!(preserved.len() > 16 * 1024);
    assert!(source_backed.len() < 128);
    assert!(preserved.len() > source_backed.len() * 100);
    assert!(
      codec
        .decode::<JsonOwner>(&source_backed)
        .unwrap()
        .value
        .is_none()
    );
  }

  #[test]
  fn preserve_policy_retains_canonical_json() {
    let value = json::object! { payload: "custom parser result" };
    let codec = CacheCodec::new(None);
    let encoded = codec
      .encode_with_json_policy(
        &JsonOwner {
          value: Some(value.clone()),
        },
        JsonArchivePolicy::Preserve,
      )
      .unwrap();
    assert_eq!(
      codec.decode::<JsonOwner>(&encoded).unwrap().value,
      Some(value)
    );
  }

  #[test]
  fn missing_json_remains_missing_under_both_policies() {
    let codec = CacheCodec::new(None);
    for policy in [
      JsonArchivePolicy::Preserve,
      JsonArchivePolicy::DeriveFromModuleSource,
    ] {
      let encoded = codec
        .encode_with_json_policy(&JsonOwner { value: None }, policy)
        .unwrap();
      assert!(codec.decode::<JsonOwner>(&encoded).unwrap().value.is_none());
    }
  }

  #[test]
  fn unknown_contexts_preserve_json_by_default() {
    let value = json::object! { payload: "ordinary cacheable context" };
    let owner = JsonOwner {
      value: Some(value.clone()),
    };
    let encoded = to_bytes(&owner, &()).unwrap();
    assert_eq!(
      from_bytes::<JsonOwner, _>(&encoded, &()).unwrap().value,
      Some(value)
    );
  }
}
