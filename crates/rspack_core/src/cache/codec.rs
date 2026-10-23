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
use rustc_hash::FxHasher;

use crate::PortableRoot;

/// How encoded bytes depend on the portable root.
#[derive(Debug, Clone, Copy)]
pub enum RootBinding {
  /// The bytes are valid under any root.
  Unbound,
  /// The bytes reference paths outside the root, so they stay valid wherever the root has the same
  /// realpath, however `context` spells it.
  Location,
  /// The bytes embed the root itself, so they are valid only where both spellings of the root
  /// match.
  Spelling,
}

#[derive(Debug)]
struct RootPatterns {
  location_hash: u64,
  spelling_hash: u64,
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
      let mut hasher = FxHasher::default();
      (RootBinding::Location as u8).hash(&mut hasher);
      portable_root.canonical_root.as_str().hash(&mut hasher);
      let location_hash = hasher.finish();
      let mut hasher = FxHasher::default();
      (RootBinding::Spelling as u8).hash(&mut hasher);
      portable_root.canonical_root.as_str().hash(&mut hasher);
      portable_root.root.as_str().hash(&mut hasher);
      let spelling_hash = hasher.finish();
      let mut patterns: Vec<Vec<u8>> = [&portable_root.root, &portable_root.canonical_root]
        .into_iter()
        .flat_map(|root| root_patterns(root.as_str()))
        .collect();
      patterns.sort_unstable();
      patterns.dedup();
      Arc::new(RootPatterns {
        location_hash,
        spelling_hash,
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

  pub fn is_portable(&self) -> bool {
    self.root_patterns.is_some()
  }

  /// Returns the root identity that bytes with `binding` are only valid under.
  pub fn bound_root(&self, binding: RootBinding) -> Option<u64> {
    let patterns = self.root_patterns.as_ref()?;
    match binding {
      RootBinding::Unbound => None,
      RootBinding::Location => Some(patterns.location_hash),
      RootBinding::Spelling => Some(patterns.spelling_hash),
    }
  }

  /// Returns whether bytes bound to `bound_root` are valid under this codec's root.
  pub fn accepts_bound_root(&self, bound_root: Option<u64>) -> bool {
    let Some(bound_root) = bound_root else {
      return true;
    };
    self.root_patterns.as_ref().is_some_and(|patterns| {
      bound_root == patterns.location_hash || bound_root == patterns.spelling_hash
    })
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
    let binding = if patterns
      .finders
      .iter()
      .any(|finder| finder.find(&bytes).is_some())
    {
      RootBinding::Spelling
    } else if outside {
      RootBinding::Location
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
