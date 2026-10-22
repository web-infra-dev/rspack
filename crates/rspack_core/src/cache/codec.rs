use std::path::Path;

use rspack_cacheable::{
  __private::rkyv::{Archive, Deserialize, Serialize, bytecheck::CheckBytes},
  Deserializer, Serializer, Validator, from_bytes, to_bytes,
};
use rspack_error::Result;
use rspack_paths::{Utf8Path, Utf8PathBuf};

/// Internal cacheable context for serialization
#[derive(Debug, Clone)]
struct Context {
  portable_project_root: Option<Utf8PathBuf>,
}

impl rspack_cacheable::CacheableContext for Context {
  fn project_root(&self) -> Option<&Path> {
    self.portable_project_root.as_ref().map(|p| p.as_std_path())
  }
}

/// Cache codec for encoding and decoding cacheable data
///
/// This struct encapsulates the serialization and deserialization logic,
/// automatically passing the project context to rspack_cacheable's to_bytes and from_bytes.
///
/// # Example
///
/// ```ignore
/// let codec = CacheCodec::new(portable_project_root);
///
/// // Encode data to bytes
/// let bytes = codec.encode(&my_data)?;
///
/// // Decode bytes back to data
/// let my_data: MyType = codec.decode(&bytes)?;
/// ```
#[derive(Debug, Clone)]
pub struct CacheCodec {
  context: Context,
}

impl CacheCodec {
  pub fn new(portable_project_root: Option<Utf8PathBuf>) -> Self {
    Self {
      context: Context {
        portable_project_root,
      },
    }
  }

  /// The cache version to store and compare on restore.
  ///
  /// Without a portable project root, encoded data holds absolute paths under `context`,
  /// so a cache moved or shared to another root must fail validation instead of
  /// restoring the other root's paths.
  pub fn validation_version(&self, version: &str, context: &Utf8Path) -> String {
    if self.context.portable_project_root.is_some() {
      version.to_string()
    } else {
      format!("{version}|{context}")
    }
  }

  pub fn encode<T>(&self, data: &T) -> Result<Vec<u8>>
  where
    T: for<'a> Serialize<Serializer<'a>>,
  {
    to_bytes(data, &self.context).map_err(rspack_error::Error::from_error)
  }

  pub fn decode<T>(&self, bytes: &[u8]) -> Result<T>
  where
    T: Archive,
    T::Archived: for<'a> CheckBytes<Validator<'a>> + Deserialize<T, Deserializer>,
  {
    from_bytes(bytes, &self.context).map_err(rspack_error::Error::from_error)
  }
}
