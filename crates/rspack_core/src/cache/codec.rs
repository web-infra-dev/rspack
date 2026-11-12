use std::{
  hash::{Hash, Hasher},
  path::Path,
  sync::Arc,
};

use aho_corasick::AhoCorasick;
use memchr::memmem;
use rspack_cacheable::{
  __private::rkyv::{Archive, Deserialize, Serialize, bytecheck::CheckBytes},
  Deserializer, Serializer, Validator, from_bytes, to_bytes, to_bytes_with_binding,
  utils::root_patterns,
};
use rspack_error::Result;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::FxHasher;

#[derive(Debug, Clone, Copy)]
pub enum RootBinding {
  Unbound,
  Bound,
}

#[derive(Debug)]
struct RootPatterns {
  hash: u64,
  matcher: RootMatcher,
  bind_all: bool,
}

/// Finds any root pattern in encoded bytes. Every entry of a portable cache is searched, so this
/// runs over all the bytes a build stores.
#[derive(Debug)]
struct RootMatcher {
  /// A substring of every pattern in `gated`: bytes without it hold none of them. The spellings
  /// of a root share its names, so one substring search stands in for most patterns.
  gate: Option<memmem::Finder<'static>>,
  gated: PatternSet,
  /// The patterns that lack the gate, such as the base64 spellings.
  ungated: PatternSet,
}

impl RootMatcher {
  fn new(patterns: &[Vec<u8>]) -> Self {
    // The run of letters and digits that the most patterns contain, and the longest of those.
    // The shortest pattern is the root as it is, whose names every escaped spelling keeps.
    let gate = patterns
      .iter()
      .min_by_key(|pattern| pattern.len())
      .into_iter()
      .flat_map(|pattern| pattern.split(|byte| !byte.is_ascii_alphanumeric()))
      .filter(|run| run.len() >= 3)
      .map(|run| {
        let count = patterns
          .iter()
          .filter(|pattern| memmem::find(pattern, run).is_some())
          .count();
        (count, run.len(), run)
      })
      .filter(|(count, ..)| *count > 1)
      .max()
      .map(|(.., run)| run);
    let (gated, ungated): (Vec<_>, Vec<_>) = patterns
      .iter()
      .partition(|pattern| gate.is_some_and(|gate| memmem::find(pattern, gate).is_some()));
    Self {
      gate: gate.map(|gate| memmem::Finder::new(gate).into_owned()),
      gated: PatternSet::new(&gated),
      ungated: PatternSet::new(&ungated),
    }
  }

  fn is_match(&self, bytes: &[u8]) -> bool {
    self.ungated.is_match(bytes)
      || (self
        .gate
        .as_ref()
        .is_some_and(|gate| gate.find(bytes).is_some())
        && self.gated.is_match(bytes))
  }
}

#[derive(Debug)]
enum PatternSet {
  /// A substring search for each pattern. While the patterns are few this outruns the automaton,
  /// whose prefilter stops at bytes that are common in source code.
  Finders(Vec<memmem::Finder<'static>>),
  Automaton(AhoCorasick),
}

impl PatternSet {
  /// A Windows root has a few dozen base64 spellings. Above that a pass for each pattern costs
  /// more than one pass of the automaton.
  const MAX_FINDERS: usize = 32;

  fn new(patterns: &[&Vec<u8>]) -> Self {
    if patterns.len() <= Self::MAX_FINDERS {
      Self::Finders(
        patterns
          .iter()
          .map(|pattern| memmem::Finder::new(pattern).into_owned())
          .collect(),
      )
    } else {
      Self::Automaton(AhoCorasick::new(patterns).expect("root patterns should build a matcher"))
    }
  }

  fn is_match(&self, bytes: &[u8]) -> bool {
    match self {
      Self::Finders(finders) => finders.iter().any(|finder| finder.find(bytes).is_some()),
      Self::Automaton(automaton) => automaton.is_match(bytes),
    }
  }
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
      let patterns = root_patterns(root.as_str());
      Arc::new(RootPatterns {
        hash: hasher.finish(),
        matcher: RootMatcher::new(&patterns.patterns),
        bind_all: patterns.bind_all,
      })
    });
    Self {
      root_patterns,
      context: Context {
        portable_project_root,
      },
    }
  }

  /// The cache version to store and compare on restore.
  ///
  /// Without a portable project root, encoded data holds absolute paths under `context`,
  /// so a cache moved or shared to another root must fail validation instead of
  /// restoring the other root's paths. Portable entries use the `BoundEntry`
  /// envelope, so the two encodings must not share a cache either.
  pub fn validation_version(&self, version: &str, context: &Utf8Path) -> String {
    if self.context.portable_project_root.is_some() {
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
    let binding = if outside || patterns.bind_all || patterns.matcher.is_match(&bytes) {
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
