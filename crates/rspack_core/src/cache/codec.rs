use std::{
  fmt::Write,
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
use rspack_paths::{Utf8Component, Utf8Path};
use rustc_hash::FxHasher;

use crate::PortableRoot;

/// How encoded bytes depend on the portable root.
#[derive(Debug, Clone, Copy)]
pub enum RootBinding {
  /// The bytes are valid under any root.
  Unbound,
  /// The bytes embed the root or reference paths outside it. Outside paths may be derived from
  /// either spelling of the root, so the bytes are valid only where both spellings match.
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
      let mut hasher = FxHasher::default();
      portable_root.canonical_root.as_str().hash(&mut hasher);
      portable_root.root.as_str().hash(&mut hasher);
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

  /// The cache version to store and compare on restore. It is the only place that decides which
  /// environments may share a cache, and both cache backends use it.
  ///
  /// Without a portable root, encoded data holds absolute paths under `context`, so a cache moved
  /// or shared to another context must fail validation instead of restoring the other context's
  /// paths.
  ///
  /// With a portable root, decoding rebases paths under the reader's root. Some cached values are
  /// relative to `context` instead of the root, such as `__dirname` with `node.__dirname: true`
  /// and CSS local identifiers. They match only where `context`, the root and its real path have
  /// the same layout, so the version records that layout without the location that contains it.
  /// Portable markers also depend on the host's path syntax, so the version records it too.
  ///
  /// Fields are length-prefixed, so a `|` in `version` or in a path cannot make two different
  /// environments render the same version.
  pub fn validation_version(&self, version: &str, context: &Utf8Path) -> String {
    let mut fields = vec![version];
    let layout;
    match &self.context.portable_root {
      None => fields.extend(["absolute", context.as_str()]),
      Some(portable_root) => {
        layout = portable_layout(&portable_root.root, &portable_root.canonical_root, context);
        fields.extend(["portable", if cfg!(windows) { "windows" } else { "posix" }]);
        fields.extend(layout.iter().map(String::as_str));
      }
    }
    let mut rendered = String::new();
    for field in fields {
      let _ = write!(rendered, "{}:{field}|", field.len());
    }
    rendered
  }

  pub fn is_portable(&self) -> bool {
    self.root_patterns.is_some()
  }

  /// Returns the root identity that bytes with `binding` are only valid under.
  pub fn bound_root(&self, binding: RootBinding) -> Option<u64> {
    let patterns = self.root_patterns.as_ref()?;
    match binding {
      RootBinding::Unbound => None,
      RootBinding::Bound => Some(patterns.hash),
    }
  }

  /// Returns whether bytes bound to `bound_root` are valid under this codec's root.
  pub fn accepts_bound_root(&self, bound_root: Option<u64>) -> bool {
    let Some(bound_root) = bound_root else {
      return true;
    };
    self
      .root_patterns
      .as_ref()
      .is_some_and(|patterns| bound_root == patterns.hash)
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

/// `root`, `canonical_root` and `context`, each as the components below their longest common
/// ancestor. Two environments with the same layout differ only in where that ancestor is, so a
/// path relative to any of the three is the same in both. Without a common ancestor, as with
/// different Windows prefixes, the absolute paths are the layout.
fn portable_layout(root: &Utf8Path, canonical_root: &Utf8Path, context: &Utf8Path) -> [String; 3] {
  let paths = [root, canonical_root, context];
  let components: Vec<Vec<Utf8Component>> = paths
    .iter()
    .map(|path| path.components().collect())
    .collect();
  let common = (0..)
    .take_while(|&index| {
      components[0].get(index).is_some_and(|first| {
        components[1..]
          .iter()
          .all(|other| other.get(index) == Some(first))
      })
    })
    .count();
  if common == 0 {
    return paths.map(|path| path.as_str().to_string());
  }
  let below = |components: &Vec<Utf8Component>| {
    let below: Vec<&str> = components[common..]
      .iter()
      .map(Utf8Component::as_str)
      .collect();
    if below.is_empty() {
      ".".to_string()
    } else {
      below.join("/")
    }
  };
  [
    below(&components[0]),
    below(&components[1]),
    below(&components[2]),
  ]
}
