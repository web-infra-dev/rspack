use std::path::Path;

use lzzzz::lz4;
use rspack_cacheable::{
  __private::rkyv::{Archive, Deserialize, Serialize, bytecheck::CheckBytes},
  Deserializer, Serializer, Validator, from_bytes, to_bytes,
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
/// Compression can be enabled for storage backends that retain serialized entries
/// without compressing them themselves.
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
  compression: bool,
}

impl CacheCodec {
  pub fn new(portable_project_root: Option<Utf8PathBuf>) -> Self {
    Self {
      context: Context {
        portable_project_root,
      },
      compression: false,
    }
  }

  pub fn with_compression(mut self) -> Self {
    self.compression = true;
    self
  }

  pub fn encode<T>(&self, data: &T) -> Result<Vec<u8>>
  where
    T: for<'a> Serialize<Serializer<'a>>,
  {
    let bytes = to_bytes(data, &self.context).map_err(rspack_error::Error::from_error)?;
    if !self.compression {
      return Ok(bytes);
    }
    // Storage retains encoded entries while writes are pending. Keep compressed
    // payloads rather than another full copy of the compilation state.
    let uncompressed_len = u32::try_from(bytes.len()).map_err(rspack_error::Error::from_error)?;
    let mut compressed = Vec::with_capacity(4 + lz4::max_compressed_size(bytes.len()));
    compressed.extend_from_slice(&uncompressed_len.to_le_bytes());
    lz4::compress_to_vec(&bytes, &mut compressed, lz4::ACC_LEVEL_DEFAULT)
      .map_err(rspack_error::Error::from_error)?;
    compressed.shrink_to_fit();
    Ok(compressed)
  }

  pub fn decode<T>(&self, bytes: &[u8]) -> Result<T>
  where
    T: Archive,
    T::Archived: for<'a> CheckBytes<Validator<'a>> + Deserialize<T, Deserializer>,
  {
    if !self.compression {
      return from_bytes(bytes, &self.context).map_err(rspack_error::Error::from_error);
    }
    let (uncompressed_len, compressed) = bytes
      .split_first_chunk::<4>()
      .ok_or_else(|| rspack_error::error!("Cache entry is missing its uncompressed length"))?;
    let uncompressed_len = u32::from_le_bytes(*uncompressed_len) as usize;
    let mut bytes = vec![0; uncompressed_len];
    // lzzzz emits no block for an empty input.
    let written = if compressed.is_empty() && uncompressed_len == 0 {
      0
    } else {
      lz4::decompress(compressed, &mut bytes).map_err(rspack_error::Error::from_error)?
    };
    if written != uncompressed_len {
      return Err(rspack_error::error!(
        "Cache entry decompressed length mismatch: expected {uncompressed_len}, got {written}"
      ));
    }
    from_bytes(&bytes, &self.context).map_err(rspack_error::Error::from_error)
  }
}
