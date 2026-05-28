use std::{
  any::{Any, TypeId},
  borrow::Cow,
  convert::{TryFrom, TryInto},
  fmt,
  hash::{Hash, Hasher},
  sync::Arc,
};

use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};

use crate::{
  Result,
  helpers::{Chunks, StreamChunks, decode_mappings},
  object_pool::ObjectPool,
};

/// An alias for `Box<dyn Source>`.
pub type BoxSource = Arc<dyn Source>;

/// A unified representation for source content that can be either text or binary data.
///
/// `SourceValue` provides a flexible way to handle source content regardless of whether
/// it's originally stored as a string or raw bytes. This is particularly useful for
/// build tools and bundlers that need to process various types of source files.
#[derive(Debug, PartialEq, Eq)]
pub enum SourceValue<'a> {
  /// Text content stored as a UTF-8 string.
  String(Cow<'a, str>),
  /// Binary content stored as raw bytes.
  Buffer(Cow<'a, [u8]>),
}

impl<'a> SourceValue<'a> {
  /// Convert the source value to a string using lossy UTF-8 conversion.
  ///
  /// This method converts both string and buffer variants to `Cow<str>`.
  /// For buffer data that contains invalid UTF-8 sequences, replacement
  /// characters (�) will be used in place of invalid sequences.
  pub fn into_string_lossy(self) -> Cow<'a, str> {
    match self {
      SourceValue::String(cow) => cow,
      SourceValue::Buffer(cow) => match cow {
        Cow::Borrowed(bytes) => String::from_utf8_lossy(bytes),
        Cow::Owned(bytes) => {
          match String::from_utf8_lossy(&bytes) {
            Cow::Borrowed(_) => {
              // SAFETY: When `String::from_utf8_lossy` returns `Cow::Borrowed(_)`,
              // it guarantees that the input slice contains only valid UTF-8 bytes.
              // Since we're operating on the exact same `bytes` that were just
              // validated by `from_utf8_lossy`, we can safely skip the UTF-8
              // validation in `String::from_utf8_unchecked`.
              //
              // This optimization avoids the redundant UTF-8 validation that would
              // occur if we used `String::from_utf8(bytes).unwrap()` or similar.
              #[allow(unsafe_code)]
              Cow::Owned(unsafe { String::from_utf8_unchecked(bytes) })
            }
            Cow::Owned(s) => Cow::Owned(s),
          }
        }
      },
    }
  }

  /// Get a reference to the source content as bytes.
  ///
  /// This method provides access to the raw byte representation of the source
  /// content regardless of whether it was originally stored as a string or buffer.
  pub fn as_bytes(&self) -> &[u8] {
    match self {
      SourceValue::String(cow) => cow.as_bytes(),
      SourceValue::Buffer(cow) => cow.as_ref(),
    }
  }

  /// Convert the source value into bytes.
  ///
  /// This method consumes the `SourceValue` and converts it to `Cow<'a, [u8]>`,
  /// providing the most efficient representation possible while preserving
  /// the original borrowing relationships.
  pub fn into_bytes(self) -> Cow<'a, [u8]> {
    match self {
      SourceValue::String(cow) => match cow {
        Cow::Borrowed(s) => Cow::Borrowed(s.as_bytes()),
        Cow::Owned(s) => Cow::Owned(s.into_bytes()),
      },
      SourceValue::Buffer(cow) => cow,
    }
  }

  /// Check if the source value contains binary data.
  ///
  /// Returns `true` if this `SourceValue` is a `Buffer` variant containing
  /// raw bytes, `false` if it's a `String` variant containing text data.
  pub fn is_buffer(&self) -> bool {
    matches!(self, SourceValue::Buffer(_))
  }

  /// Returns `true` if `self` has a length of zero bytes.
  pub fn is_empty(&self) -> bool {
    match self {
      SourceValue::String(string) => string.is_empty(),
      SourceValue::Buffer(buffer) => buffer.is_empty(),
    }
  }
}

/// [Source] abstraction, [webpack-sources docs](https://github.com/webpack/webpack-sources/#source).
pub trait Source:
  StreamChunks + DynHash + AsAny + DynEq + DynClone + fmt::Debug + Sync + Send
{
  /// Get the source code.
  fn source(&self) -> SourceValue<'_>;

  /// Return a lightweight "rope" view of the source as borrowed string slices.
  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str));

  /// Get the source buffer.
  fn buffer(&self) -> Cow<'_, [u8]>;

  /// Get the size of the source.
  fn size(&self) -> usize;

  /// Get the [SourceMap].
  fn map(&self, object_pool: &ObjectPool, options: &MapOptions) -> Option<SourceMap>;

  /// Update hash based on the source.
  fn update_hash(&self, state: &mut dyn Hasher) {
    self.dyn_hash(state);
  }

  /// Writes the source into a writer, preferably a `std::io::BufWriter<std::io::Write>`.
  fn to_writer(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()>;
}

impl Source for BoxSource {
  #[inline]
  fn source(&self) -> SourceValue<'_> {
    self.as_ref().source()
  }

  #[inline]
  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    self.as_ref().rope(on_chunk)
  }

  #[inline]
  fn buffer(&self) -> Cow<'_, [u8]> {
    self.as_ref().buffer()
  }

  #[inline]
  fn size(&self) -> usize {
    self.as_ref().size()
  }

  #[inline]
  fn map(&self, object_pool: &ObjectPool, options: &MapOptions) -> Option<SourceMap> {
    self.as_ref().map(object_pool, options)
  }

  #[inline]
  fn to_writer(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    self.as_ref().to_writer(writer)
  }
}

dyn_clone::clone_trait_object!(Source);

impl StreamChunks for BoxSource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks + 'a> {
    self.as_ref().stream_chunks()
  }
}

// for `updateHash`
pub trait DynHash {
  fn dyn_hash(&self, state: &mut dyn Hasher);
}

impl<H: Hash> DynHash for H {
  fn dyn_hash(&self, mut state: &mut dyn Hasher) {
    self.hash(&mut state);
  }
}

impl Hash for dyn Source {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.dyn_hash(state)
  }
}

pub trait AsAny {
  fn as_any(&self) -> &dyn Any;
}

impl<T: Any> AsAny for T {
  fn as_any(&self) -> &dyn Any {
    self
  }
}

pub trait DynEq {
  fn dyn_eq(&self, other: &dyn Any) -> bool;
  fn type_id(&self) -> TypeId;
}

impl<E: Eq + Any> DynEq for E {
  fn dyn_eq(&self, other: &dyn Any) -> bool {
    if let Some(other) = other.downcast_ref::<E>() {
      self == other
    } else {
      false
    }
  }

  fn type_id(&self) -> TypeId {
    TypeId::of::<E>()
  }
}

impl PartialEq for dyn Source {
  fn eq(&self, other: &Self) -> bool {
    if self.as_any().type_id() != other.as_any().type_id() {
      return false;
    }
    self.dyn_eq(other.as_any())
  }
}

impl Eq for dyn Source {}

/// Extension methods for [Source].
pub trait SourceExt {
  /// An alias for [BoxSource::from].
  fn boxed(self) -> BoxSource;
}

impl<T: Source + 'static> SourceExt for T {
  fn boxed(self) -> BoxSource {
    if let Some(source) = self.as_any().downcast_ref::<BoxSource>() {
      return source.clone();
    }
    Arc::new(self)
  }
}

/// Options for [Source::map].
#[derive(Debug, Clone)]
pub struct MapOptions {
  /// Whether have columns info in generated [SourceMap] mappings.
  pub columns: bool,
  /// Whether the source will have changes, internal used for `ReplaceSource`, etc.
  pub(crate) final_source: bool,
}

impl Default for MapOptions {
  fn default() -> Self {
    Self {
      columns: true,
      final_source: false,
    }
  }
}

impl MapOptions {
  /// Create [MapOptions] with columns.
  pub fn new(columns: bool) -> Self {
    Self {
      columns,
      ..Default::default()
    }
  }
}

fn is_all_empty(val: &[Arc<str>]) -> bool {
  if val.is_empty() {
    return true;
  }
  val.iter().all(|s| s.is_empty())
}

/// The source map created by [Source::map].
#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct SourceMap {
  version: u8,
  #[serde(skip_serializing_if = "Option::is_none")]
  file: Option<Arc<str>>,
  sources: Arc<[String]>,
  #[serde(rename = "sourcesContent", skip_serializing_if = "is_all_empty")]
  sources_content: Arc<[Arc<str>]>,
  names: Arc<[String]>,
  mappings: Arc<str>,
  #[serde(rename = "sourceRoot", skip_serializing_if = "Option::is_none")]
  source_root: Option<Arc<str>>,
  #[serde(rename = "debugId", skip_serializing_if = "Option::is_none")]
  debug_id: Option<Arc<str>>,
  #[serde(rename = "ignoreList", skip_serializing_if = "Option::is_none")]
  ignore_list: Option<Arc<Vec<u32>>>,
}

impl std::fmt::Debug for SourceMap {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    let indent = f.width().unwrap_or(0);
    let indent_str = format!("{:indent$}", "", indent = indent);

    write!(
      f,
      "{indent_str}SourceMap::from_json({:?}).unwrap()",
      self.clone().to_json()
    )?;

    Ok(())
  }
}
impl Hash for SourceMap {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.file.hash(state);
    self.mappings.hash(state);
    self.sources.hash(state);
    self.sources_content.hash(state);
    self.names.hash(state);
    self.source_root.hash(state);
    self.ignore_list.hash(state);
  }
}

impl SourceMap {
  /// Create a [SourceMap].
  pub fn new<Mappings, Sources, SourcesContent, Names>(
    mappings: Mappings,
    sources: Sources,
    sources_content: SourcesContent,
    names: Names,
  ) -> Self
  where
    Mappings: Into<Arc<str>>,
    Sources: Into<Arc<[String]>>,
    SourcesContent: Into<Vec<Arc<str>>>,
    Names: Into<Arc<[String]>>,
  {
    Self {
      version: 3,
      file: None,
      mappings: mappings.into(),
      sources: sources.into(),
      sources_content: Arc::from(sources_content.into()),
      names: names.into(),
      source_root: None,
      debug_id: None,
      ignore_list: None,
    }
  }

  /// Get the file field in [SourceMap].
  pub fn file(&self) -> Option<&str> {
    self.file.as_deref()
  }

  /// Set the file field in [SourceMap].
  pub fn set_file<T: Into<Arc<str>>>(&mut self, file: Option<T>) {
    self.file = file.map(Into::into);
  }

  /// Get the ignoreList field in [SourceMap].
  pub fn ignore_list(&self) -> Option<&[u32]> {
    self.ignore_list.as_deref().map(|v| &**v)
  }

  /// Set the ignoreList field in [SourceMap].
  pub fn set_ignore_list<T: Into<Vec<u32>>>(&mut self, ignore_list: Option<T>) {
    self.ignore_list = ignore_list.map(|v| Arc::new(v.into()));
  }

  /// Get the decoded mappings in [SourceMap].
  pub fn decoded_mappings(&self) -> impl Iterator<Item = Mapping> + '_ {
    decode_mappings(self)
  }

  /// Get the mappings string in [SourceMap].
  pub fn mappings(&self) -> &str {
    &self.mappings
  }

  /// Get the sources field in [SourceMap].
  pub fn sources(&self) -> &[String] {
    &self.sources
  }

  /// Set the sources field in [SourceMap].
  pub fn set_sources<T: Into<Arc<[String]>>>(&mut self, sources: T) {
    self.sources = sources.into();
  }

  /// Get the source by index from sources field in [SourceMap].
  pub fn get_source(&self, index: usize) -> Option<&str> {
    self.sources.get(index).map(|s| s.as_ref())
  }

  /// Get the sourcesContent field in [SourceMap].
  pub fn sources_content(&self) -> &[Arc<str>] {
    &self.sources_content
  }

  /// Set the sourcesContent field in [SourceMap].
  pub fn set_sources_content<T: Into<Vec<Arc<str>>>>(&mut self, sources_content: T) {
    self.sources_content = Arc::from(sources_content.into());
  }

  /// Get the source content by index from sourcesContent field in [SourceMap].
  pub fn get_source_content(&self, index: usize) -> Option<&Arc<str>> {
    self.sources_content.get(index)
  }

  /// Get the names field in [SourceMap].
  pub fn names(&self) -> &[String] {
    &self.names
  }

  /// Set the names field in [SourceMap].
  pub fn set_names<T: Into<Arc<[String]>>>(&mut self, names: T) {
    self.names = names.into();
  }

  /// Get the name by index from names field in [SourceMap].
  pub fn get_name(&self, index: usize) -> Option<&str> {
    self.names.get(index).map(|s| s.as_ref())
  }

  /// Get the source_root field in [SourceMap].
  pub fn source_root(&self) -> Option<&str> {
    self.source_root.as_deref()
  }

  /// Set the source_root field in [SourceMap].
  pub fn set_source_root<T: Into<Arc<str>>>(&mut self, source_root: Option<T>) {
    self.source_root = source_root.map(Into::into);
  }

  /// Set the debug_id field in [SourceMap].
  pub fn set_debug_id<T: Into<Arc<str>>>(&mut self, debug_id: Option<T>) {
    self.debug_id = debug_id.map(Into::into);
  }

  /// Get the debug_id field in [SourceMap].
  pub fn get_debug_id(&self) -> Option<&str> {
    self.debug_id.as_deref()
  }
}

#[derive(Debug, Default, Deserialize)]
struct RawSourceMap {
  pub file: Option<String>,
  pub sources: Option<Vec<Option<String>>>,
  #[serde(rename = "sourceRoot")]
  pub source_root: Option<String>,
  #[serde(rename = "sourcesContent")]
  pub sources_content: Option<Vec<Option<String>>>,
  pub names: Option<Vec<Option<String>>>,
  pub mappings: String,
  #[serde(rename = "debugId")]
  pub debug_id: Option<String>,
  #[serde(rename = "ignoreList")]
  pub ignore_list: Option<Vec<u32>>,
}

impl RawSourceMap {
  pub fn from_reader<R: std::io::Read>(r: R) -> Result<Self> {
    let raw: RawSourceMap = simd_json::serde::from_reader(r)?;
    Ok(raw)
  }

  pub fn from_slice(val: &[u8]) -> Result<Self> {
    let mut v = val.to_vec();
    let raw: RawSourceMap = simd_json::serde::from_slice(&mut v)?;
    Ok(raw)
  }

  pub fn from_json(val: &str) -> Result<Self> {
    let mut v = val.as_bytes().to_vec();
    let raw: RawSourceMap = simd_json::serde::from_slice(&mut v)?;
    Ok(raw)
  }
}

impl SourceMap {
  /// Create a [SourceMap] from json string.
  pub fn from_json(s: &str) -> Result<Self> {
    RawSourceMap::from_json(s)?.try_into()
  }

  /// Create a [SourceMap] from [&[u8]].
  pub fn from_slice(s: &[u8]) -> Result<Self> {
    RawSourceMap::from_slice(s)?.try_into()
  }

  /// Create a [SourceMap] from reader.
  pub fn from_reader<R: std::io::Read>(s: R) -> Result<Self> {
    RawSourceMap::from_reader(s)?.try_into()
  }

  /// Estimate the JSON string size for pre-allocation.
  ///
  /// This estimation aims to be accurate in ~90% of cases to avoid reallocation.
  /// The formula accounts for:
  /// - Fixed overhead: `{"version":3,"sources":[],"names":[],"mappings":""}`
  /// - Per-field and per-element lengths, including commas and quotes
  /// - Extra escaping overhead for `sourcesContent` and a ~10% overall safety margin
  #[inline]
  fn json_size_hint(&self) -> usize {
    // Base structure overhead:
    // {"version":3,"sources":[],"sourcesContent":[],"names":[],"mappings":""}
    // ≈ 70 bytes base + optional fields
    let mut size: usize = 70;

    // file field: "file":"...",
    if let Some(file) = &self.file {
      size += 9 + file.len(); // "file":"", + content
    }

    // sources array: each element needs quotes + comma + potential escaping
    // ["src/a.js","src/b.js"] = 2 + (len + 3) * count - 1
    let sources_len: usize = self.sources.iter().map(|s| s.len()).sum();
    size += 2 + sources_len + self.sources.len() * 3;

    // sourcesContent array
    if !self.sources_content.is_empty() {
      let content_len: usize = self.sources_content.iter().map(|c| c.len()).sum();
      // Source content often contains special characters that need escaping
      // Estimate 10% escaping overhead for source content
      size += 19 + content_len + (content_len / 10) + self.sources_content.len() * 3;
    }

    // names array
    let names_len: usize = self.names.iter().map(|n| n.len()).sum();
    size += 2 + names_len + self.names.len() * 3;

    // mappings string (usually the largest part)
    // VLQ mappings rarely need escaping, add small overhead
    size += self.mappings.len() + 14; // "mappings":"...",

    // sourceRoot field
    if let Some(source_root) = &self.source_root {
      size += 15 + source_root.len(); // "sourceRoot":"...",
    }

    // debugId field
    if let Some(debug_id) = &self.debug_id {
      size += 12 + debug_id.len(); // "debugId":"...",
    }

    // ignoreList field: [0,1,2] - numbers as strings
    if let Some(ignore_list) = &self.ignore_list {
      // "ignoreList":[]
      size += 14;
      // Each number: up to 10 digits + comma
      size += ignore_list.len() * 6;
    }

    // Add 10% safety margin to handle edge cases (escaping, larger numbers, etc.)
    size + size / 10
  }

  /// Generate source map to a json string.
  pub fn to_json(&self) -> String {
    let mut buffer = Vec::with_capacity(self.json_size_hint());

    simd_json::to_writer(&mut buffer, self).unwrap();

    // SAFETY: simd_json always produces valid UTF-8 JSON
    #[allow(unsafe_code)]
    unsafe {
      String::from_utf8_unchecked(buffer)
    }
  }
}

impl TryFrom<RawSourceMap> for SourceMap {
  type Error = crate::Error;

  fn try_from(raw: RawSourceMap) -> Result<Self> {
    let file = raw.file.map(Into::into);
    let mappings = raw.mappings.into();
    let sources = raw
      .sources
      .unwrap_or_default()
      .into_iter()
      .map(Option::unwrap_or_default)
      .collect::<Vec<_>>()
      .into();
    let sources_content = raw
      .sources_content
      .unwrap_or_default()
      .into_iter()
      .map(|source_content| Arc::from(source_content.unwrap_or_default()))
      .collect::<Vec<_>>()
      .into();
    let names = raw
      .names
      .unwrap_or_default()
      .into_iter()
      .map(Option::unwrap_or_default)
      .collect::<Vec<_>>()
      .into();
    let source_root = raw.source_root.map(Into::into);
    let debug_id = raw.debug_id.map(Into::into);
    let ignore_list = raw.ignore_list.map(Into::into);

    Ok(Self {
      version: 3,
      file,
      mappings,
      sources,
      sources_content,
      names,
      source_root,
      debug_id,
      ignore_list,
    })
  }
}

/// Represent a [Mapping] information of source map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mapping {
  /// Generated line.
  pub generated_line: u32,
  /// Generated column.
  pub generated_column: u32,
  /// Original position information.
  pub original: Option<OriginalLocation>,
}

/// Represent original position information of a [Mapping].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OriginalLocation {
  /// Source index.
  pub source_index: u32,
  /// Original line.
  pub original_line: u32,
  /// Original column.
  pub original_column: u32,
  /// Name index.
  pub name_index: Option<u32>,
}

/// An convenient way to create a [Mapping].
#[macro_export]
macro_rules! m {
  ($gl:expr, $gc:expr, $si:expr, $ol:expr, $oc:expr, $ni:expr) => {{
    let gl: i64 = $gl;
    let gc: i64 = $gc;
    let si: i64 = $si;
    let ol: i64 = $ol;
    let oc: i64 = $oc;
    let ni: i64 = $ni;
    $crate::Mapping {
      generated_line: gl as u32,
      generated_column: gc as u32,
      original: (si >= 0).then(|| $crate::OriginalLocation {
        source_index: si as u32,
        original_line: ol as u32,
        original_column: oc as u32,
        name_index: (ni >= 0).then(|| ni as u32),
      }),
    }
  }};
}

/// An convenient way to create [Mapping]s.
#[macro_export]
macro_rules! mappings {
  ($($mapping:expr),* $(,)?) => {
    ::std::vec![$({
      let mapping = $mapping;
      $crate::m![mapping[0], mapping[1], mapping[2], mapping[3], mapping[4], mapping[5]]
    }),*]
  };
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use super::*;
  use crate::{
    CachedSource, ConcatSource, OriginalSource, RawBufferSource, RawStringSource, ReplaceSource,
    SourceMapSource, WithoutOriginalOptions,
  };

  #[test]
  fn should_not_have_sources_content_field_when_it_is_empty() {
    let map = SourceMap::new(
      ";;",
      vec!["a.js".into()],
      vec!["".into(), "".into(), "".into()],
      vec!["".into(), "".into()],
    )
    .to_json();
    assert!(!map.contains("sourcesContent"));
  }

  #[test]
  fn hash_available() {
    let mut state = twox_hash::XxHash64::default();
    RawStringSource::from("a").hash(&mut state);
    OriginalSource::new("b", "").hash(&mut state);
    SourceMapSource::new(WithoutOriginalOptions {
      value: "c",
      name: "",
      source_map: SourceMap::from_json("{\"mappings\": \";\"}").unwrap(),
    })
    .hash(&mut state);
    ConcatSource::new([RawStringSource::from("d")]).hash(&mut state);
    CachedSource::new(RawStringSource::from("e")).hash(&mut state);
    ReplaceSource::new(RawStringSource::from("f")).hash(&mut state);
    RawStringSource::from("g").boxed().hash(&mut state);
    RawStringSource::from_static("a").hash(&mut state);
    RawBufferSource::from("a".as_bytes()).hash(&mut state);
    (&RawStringSource::from("h") as &dyn Source).hash(&mut state);
    ReplaceSource::new(RawStringSource::from("i").boxed()).hash(&mut state);
    assert_eq!(format!("{:x}", state.finish()), "1f41150b3252c34b");
  }

  #[test]
  fn eq_available() {
    assert_eq!(RawStringSource::from("a"), RawStringSource::from("a"));
    assert_eq!(
      RawStringSource::from_static("a"),
      RawStringSource::from_static("a")
    );
    assert_eq!(
      RawBufferSource::from("a".as_bytes()),
      RawBufferSource::from("a".as_bytes())
    );
    assert_eq!(OriginalSource::new("b", ""), OriginalSource::new("b", ""));
    assert_eq!(
      SourceMapSource::new(WithoutOriginalOptions {
        value: "c",
        name: "",
        source_map: SourceMap::from_json("{\"mappings\": \";\"}").unwrap(),
      }),
      SourceMapSource::new(WithoutOriginalOptions {
        value: "c",
        name: "",
        source_map: SourceMap::from_json("{\"mappings\": \";\"}").unwrap(),
      })
    );
    assert_eq!(
      ConcatSource::new([RawStringSource::from("d")]),
      ConcatSource::new([RawStringSource::from("d")])
    );
    assert_eq!(
      CachedSource::new(RawStringSource::from("e")),
      CachedSource::new(RawStringSource::from("e"))
    );
    assert_eq!(
      ReplaceSource::new(RawStringSource::from("f")),
      ReplaceSource::new(RawStringSource::from("f"))
    );
    assert_eq!(
      &RawStringSource::from("g").boxed(),
      &RawStringSource::from("g").boxed()
    );
    assert_eq!(
      (&RawStringSource::from("h") as &dyn Source),
      (&RawStringSource::from("h") as &dyn Source)
    );
    assert_eq!(
      ReplaceSource::new(RawStringSource::from("i").boxed()),
      ReplaceSource::new(RawStringSource::from("i").boxed())
    );
    assert_eq!(
      CachedSource::new(RawStringSource::from("j").boxed()),
      CachedSource::new(RawStringSource::from("j").boxed())
    );
  }

  #[test]
  #[allow(suspicious_double_ref_op)]
  fn clone_available() {
    let a = RawStringSource::from("a");
    assert_eq!(a, a.clone());
    let b = OriginalSource::new("b", "");
    assert_eq!(b, b.clone());
    let c = SourceMapSource::new(WithoutOriginalOptions {
      value: "c",
      name: "",
      source_map: SourceMap::from_json("{\"mappings\": \";\"}").unwrap(),
    });
    assert_eq!(c, c.clone());
    let d = ConcatSource::new([RawStringSource::from("d")]);
    assert_eq!(d, d.clone());
    let e = CachedSource::new(RawStringSource::from("e"));
    assert_eq!(e, e.clone());
    let f = ReplaceSource::new(RawStringSource::from("f"));
    assert_eq!(f, f.clone());
    let g = RawStringSource::from("g").boxed();
    assert_eq!(&g, &g.clone());
    let h = &RawStringSource::from("h") as &dyn Source;
    assert_eq!(h, h);
    let i = ReplaceSource::new(RawStringSource::from("i").boxed());
    assert_eq!(i, i.clone());
    let j = CachedSource::new(RawStringSource::from("j").boxed());
    assert_eq!(j, j.clone());
    let k = RawStringSource::from_static("k");
    assert_eq!(k, k.clone());
    let l = RawBufferSource::from("l".as_bytes());
    assert_eq!(l, l.clone());
  }

  #[test]
  fn box_dyn_source_use_hashmap_available() {
    let mut map = HashMap::new();
    let a = RawStringSource::from("a").boxed();
    map.insert(a.clone(), a.clone());
    assert_eq!(map.get(&a).unwrap(), &a);
  }

  #[test]
  #[allow(suspicious_double_ref_op)]
  fn ref_dyn_source_use_hashmap_available() {
    let mut map = HashMap::new();
    let a = &RawStringSource::from("a") as &dyn Source;
    map.insert(a, a);
    assert_eq!(map.get(&a).unwrap(), &a);
  }

  #[test]
  fn to_writer() {
    let sources = ConcatSource::new([RawStringSource::from("a"), RawStringSource::from("b")]);
    let mut writer = std::io::BufWriter::new(Vec::new());
    let result = sources.to_writer(&mut writer);
    assert!(result.is_ok());
    assert_eq!(
      String::from_utf8(writer.into_inner().unwrap()).unwrap(),
      "ab"
    );
  }
}
