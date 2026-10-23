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
use rspack_paths::Utf8Path;
use rustc_hash::FxHasher;

use crate::PortableRoot;

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
  portable_root: Option<PortableRoot>,
}

impl rspack_cacheable::CacheableContext for Context {
  fn project_root(&self) -> Option<&Path> {
    self.portable_root.as_ref().map(|p| p.root.as_std_path())
  }

  fn canonical_project_root(&self) -> Option<&Path> {
    self
      .portable_root
      .as_ref()
      .map(|p| p.canonical_root.as_std_path())
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
/// let codec = CacheCodec::new(portable_root);
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
  pub fn new(portable_root: Option<PortableRoot>) -> Self {
    let root_patterns = portable_root.as_ref().map(|portable_root| {
      // A bound entry may embed either spelling, so it is only valid where both match.
      let mut hasher = FxHasher::default();
      portable_root.root.as_str().as_bytes().hash(&mut hasher);
      portable_root.canonical_root.as_str().as_bytes().hash(&mut hasher);
      let mut patterns: Vec<Vec<u8>> = [&portable_root.root, &portable_root.canonical_root]
        .into_iter()
        .flat_map(|root| root_patterns(root.as_str()))
        .collect();
      patterns.sort_unstable();
      patterns.dedup();
      Arc::new(RootPatterns {
        hash: hasher.finish(),
        finders: patterns
          .iter()
          .map(|pattern| Finder::new(pattern).into_owned())
          .collect(),
      })
    });
    Self {
      root_patterns,
      context: Context { portable_root },
    }
  }

  /// The cache version to store and compare on restore.
  ///
  /// Without a portable project root, encoded data holds absolute paths under `context`,
  /// so a cache moved or shared to another root must fail validation instead of
  /// restoring the other root's paths. Portable entries use the `BoundEntry`
  /// envelope, so the two encodings must not share a cache either.
  pub fn validation_version(&self, version: &str, context: &Utf8Path) -> String {
    if self.context.portable_root.is_some() {
      format!("{version}|portable")
    } else {
      format!("{version}|{context}")
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
