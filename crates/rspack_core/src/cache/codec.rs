mod lz4;

use std::path::Path;

use rspack_cacheable::{
  __private::rkyv::{Archive, Deserialize, Serialize, bytecheck::CheckBytes},
  Deserializer, Serializer, Validator, from_bytes, to_aligned_bytes, to_bytes,
};
use rspack_error::Result;
use rspack_paths::Utf8PathBuf;

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
  value_compression: bool,
}

impl CacheCodec {
  pub fn new(portable_project_root: Option<Utf8PathBuf>) -> Self {
    Self {
      context: Context {
        portable_project_root,
      },
      value_compression: false,
    }
  }

  pub fn with_value_compression(mut self, enabled: bool) -> Self {
    self.value_compression = enabled;
    self
  }

  pub(crate) fn value_compression_enabled(&self) -> bool {
    self.value_compression
  }

  pub fn encode_value<T>(&self, data: &T) -> Result<Vec<u8>>
  where
    T: for<'a> Serialize<Serializer<'a>>,
  {
    if !self.value_compression {
      return self.encode(data);
    }
    let bytes = to_aligned_bytes(data, &self.context).map_err(rspack_error::Error::from_error)?;
    lz4::encode(bytes)
  }

  /// Takes the loaded bytes by value so that, with compression on, they are
  /// freed once decompressed, before deserialization allocates the value.
  pub fn decode_value<T>(&self, bytes: Vec<u8>) -> Result<T>
  where
    T: Archive,
    T::Archived: for<'a> CheckBytes<Validator<'a>> + Deserialize<T, Deserializer>,
  {
    if !self.value_compression {
      return self.decode(&bytes);
    }
    let aligned = lz4::decode(&bytes)?;
    drop(bytes);
    self.decode(aligned.as_slice())
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
