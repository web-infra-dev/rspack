use std::{
  hash::{Hash, Hasher},
  path::Path,
  sync::Arc,
};

use memchr::memmem::Finder;
use rspack_cacheable::{
  __private::rkyv::{Archive, Deserialize, Serialize, bytecheck::CheckBytes},
  Deserializer, Serializer, Validator, from_bytes, to_bytes, to_bytes_with_binding,
  utils::root_patterns,
};
use rspack_error::Result;
use rspack_paths::Utf8PathBuf;
use rustc_hash::FxHasher;

#[derive(Debug, Clone, Copy)]
pub enum RootBinding {
  Unbound,
  Bound,
}

#[derive(Debug)]
struct RootPatterns {
  hash: u64,
  finders: Vec<Finder<'static>>,
}

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
  root_patterns: Option<Arc<RootPatterns>>,
}

impl CacheCodec {
  pub fn new(portable_project_root: Option<Utf8PathBuf>) -> Self {
    let root_patterns = portable_project_root.as_ref().map(|root| {
      let mut hasher = FxHasher::default();
      root.as_str().as_bytes().hash(&mut hasher);
      Arc::new(RootPatterns {
        hash: hasher.finish(),
        finders: root_patterns(root.as_str())
          .iter()
          .map(|pattern| Finder::new(pattern).into_owned())
          .collect(),
      })
    });
    Self {
      root_patterns,
      context: Context {
        portable_project_root,
      },
    }
  }

  pub fn root_hash(&self) -> Option<u64> {
    self.root_patterns.as_ref().map(|patterns| patterns.hash)
  }

  pub fn encode_with_binding<T>(&self, data: &T) -> Result<(Vec<u8>, RootBinding)>
  where
    T: for<'a> Serialize<Serializer<'a>>,
  {
    let Some(patterns) = &self.root_patterns else {
      return self.encode(data).map(|bytes| (bytes, RootBinding::Unbound));
    };
    let (bytes, outside) =
      to_bytes_with_binding(data, &self.context).map_err(rspack_error::Error::from_error)?;
    let binding = if outside
      || patterns
        .finders
        .iter()
        .any(|finder| finder.find(&bytes).is_some())
    {
      RootBinding::Bound
    } else {
      RootBinding::Unbound
    };
    Ok((bytes, binding))
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
