use std::{
  any::{Any, TypeId},
  borrow::Cow,
  fmt,
  hash::{Hash, Hasher},
  sync::Arc,
};

use serde::{Serialize, Serializer};
use simd_json::{BorrowedValue, ErrorType, prelude::*};

use crate::{
  Result,
  helpers::{Chunks, StreamChunks, decode_mappings_fields},
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
pub trait Source: StreamChunks + DynHash + AsAny + DynEq + fmt::Debug + Sync + Send {
  /// Get the source code.
  fn source(&self) -> SourceValue<'_>;

  /// Return a lightweight "rope" view of the source as borrowed string slices.
  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str));

  /// Get the source buffer.
  fn buffer(&self) -> Cow<'_, [u8]>;

  /// Get the size of the source.
  fn size(&self) -> usize;

  /// Get the [SourceMap].
  fn map<'a>(&'a self, object_pool: &ObjectPool, options: &MapOptions) -> Option<SourceMap<'a>>;

  /// Get a [SourceMap] that can outlive the borrowed source reference.
  fn map_static(
    self: Arc<Self>,
    object_pool: &ObjectPool,
    options: &MapOptions,
  ) -> Option<SourceMap<'static>>;

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
  fn map<'a>(&'a self, object_pool: &ObjectPool, options: &MapOptions) -> Option<SourceMap<'a>> {
    self.as_ref().map(object_pool, options)
  }

  #[inline]
  fn map_static(
    self: Arc<Self>,
    object_pool: &ObjectPool,
    options: &MapOptions,
  ) -> Option<SourceMap<'static>> {
    self.as_ref().clone().map_static(object_pool, options)
  }

  #[inline]
  fn to_writer(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    self.as_ref().to_writer(writer)
  }
}

impl StreamChunks for BoxSource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a> {
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
    Arc::from(self)
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

fn is_all_empty(val: &[Cow<'_, str>]) -> bool {
  if val.is_empty() {
    return true;
  }
  val.iter().all(|s| s.is_empty())
}

/// The source map created by [Source::map].
#[derive(Serialize)]
pub(crate) struct SourceMapFields<'a> {
  pub(crate) version: u8,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) file: Option<Cow<'a, str>>,
  pub(crate) sources: Cow<'a, [Cow<'a, str>]>,
  #[serde(rename = "sourcesContent", skip_serializing_if = "is_all_empty")]
  pub(crate) sources_content: Cow<'a, [Cow<'a, str>]>,
  pub(crate) names: Cow<'a, [Cow<'a, str>]>,
  pub(crate) mappings: Cow<'a, str>,
  #[serde(rename = "sourceRoot", skip_serializing_if = "Option::is_none")]
  pub(crate) source_root: Option<Cow<'a, str>>,
  #[serde(rename = "debugId", skip_serializing_if = "Option::is_none")]
  pub(crate) debug_id: Option<Cow<'a, str>>,
  #[serde(rename = "ignoreList", skip_serializing_if = "Option::is_none")]
  pub(crate) ignore_list: Option<Cow<'a, [u32]>>,
}

impl<'a> SourceMapFields<'a> {
  /// Return a source map view borrowing all fields from this map.
  pub(crate) fn as_borrowed(&self) -> SourceMapFields<'_> {
    SourceMapFields {
      version: self.version,
      file: self.file.as_ref().map(|f| Cow::Borrowed(f.as_ref())),
      sources: Cow::Borrowed(self.sources.as_ref()),
      sources_content: Cow::Borrowed(self.sources_content.as_ref()),
      names: Cow::Borrowed(self.names.as_ref()),
      mappings: Cow::Borrowed(self.mappings.as_ref()),
      source_root: self.source_root.as_ref().map(|s| Cow::Borrowed(s.as_ref())),
      debug_id: self.debug_id.as_ref().map(|s| Cow::Borrowed(s.as_ref())),
      ignore_list: self.ignore_list.as_ref().map(|s| Cow::Borrowed(s.as_ref())),
    }
  }

  pub(crate) fn mappings(&self) -> &str {
    self.mappings.as_ref()
  }

  pub(crate) fn source_root(&self) -> Option<&str> {
    self.source_root.as_deref()
  }

  pub(crate) fn sources(&self) -> &[Cow<'a, str>] {
    self.sources.as_ref()
  }

  pub(crate) fn get_source_content(&self, index: usize) -> Option<&Cow<'a, str>> {
    self.sources_content.get(index)
  }

  pub(crate) fn names(&self) -> &[Cow<'a, str>] {
    self.names.as_ref()
  }

  pub(crate) fn decoded_mappings(&self) -> impl Iterator<Item = Mapping> + '_ {
    decode_mappings_fields(self)
  }
}

impl<'a, 'b> PartialEq<SourceMapFields<'b>> for SourceMapFields<'a> {
  fn eq(&self, other: &SourceMapFields<'b>) -> bool {
    self.version == other.version
      && self.file == other.file
      && self.sources == other.sources
      && self.sources_content == other.sources_content
      && self.names == other.names
      && self.mappings == other.mappings
      && self.source_root == other.source_root
      && self.debug_id == other.debug_id
      && self.ignore_list == other.ignore_list
  }
}

impl Eq for SourceMapFields<'_> {}

impl Hash for SourceMapFields<'_> {
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

#[allow(dead_code)]
enum SourceMapOwner {
  Bytes(Vec<u8>),
  Source(BoxSource),
}

/// The source map created by [Source::map].
pub struct SourceMap<'a> {
  // Kept to retain data borrowed by `fields`; it is intentionally not read.
  #[allow(dead_code)]
  owner: Option<SourceMapOwner>,
  fields: SourceMapFields<'a>,
}

impl<'a> SourceMap<'a> {
  pub(crate) fn from_fields(fields: SourceMapFields<'a>) -> Self {
    Self {
      owner: None,
      fields,
    }
  }

  pub(crate) fn into_static(self, owner: BoxSource) -> SourceMap<'static> {
    #[allow(unsafe_code)]
    // SAFETY: `fields` must borrow from `owner` or contain owned data. The
    // returned SourceMap stores `owner`, keeping borrowed fields alive for the
    // lifetime of the SourceMap.
    let fields =
      unsafe { std::mem::transmute::<SourceMapFields<'a>, SourceMapFields<'static>>(self.fields) };
    SourceMap {
      owner: Some(SourceMapOwner::Source(owner)),
      fields,
    }
  }

  /// Return a source map view borrowing all fields from this source map.
  pub fn as_borrowed(&self) -> SourceMap<'_> {
    SourceMap {
      owner: None,
      fields: self.fields.as_borrowed(),
    }
  }

  pub(crate) fn fields(&self) -> &SourceMapFields<'a> {
    &self.fields
  }
}

impl SourceMap<'static> {
  /// Create a [SourceMap].
  pub fn new(
    mappings: impl Into<Cow<'static, str>>,
    sources: Vec<Cow<'static, str>>,
    sources_content: Vec<Cow<'static, str>>,
    names: Vec<Cow<'static, str>>,
  ) -> Self {
    Self {
      owner: None,
      fields: SourceMapFields {
        version: 3,
        file: None,
        mappings: mappings.into(),
        sources: Cow::Owned(sources),
        sources_content: Cow::Owned(sources_content),
        names: Cow::Owned(names),
        source_root: None,
        debug_id: None,
        ignore_list: None,
      },
    }
  }

  /// Create a [SourceMap] from bytes.
  pub fn from_bytes(mut bytes: Vec<u8>) -> Result<Self> {
    let fields = {
      let borrowed_value = simd_json::to_borrowed_value(bytes.as_mut_slice())?;
      let fields = deserialize_source_map_fields(&borrowed_value)?;
      #[allow(unsafe_code)]
      // SAFETY: All borrowed strings in `fields` point into the stable backing
      // allocation of `bytes`; the returned SourceMap stores `bytes` in
      // SourceMapOwner::Bytes.
      unsafe {
        std::mem::transmute::<SourceMapFields<'_>, SourceMapFields<'static>>(fields)
      }
    };
    Ok(Self {
      owner: Some(SourceMapOwner::Bytes(bytes)),
      fields,
    })
  }

  /// Create a [SourceMap] from json string.
  pub fn from_json(s: String) -> Result<Self> {
    Self::from_bytes(s.into_bytes())
  }
}

impl<'a> SourceMap<'a> {
  /// Get the file field in [SourceMap].
  pub fn file(&self) -> Option<&str> {
    self.fields.file.as_deref()
  }

  /// Set the file field in [SourceMap].
  pub fn set_file(&mut self, file: Option<Cow<'a, str>>) {
    self.fields.file = file.map(|file| Cow::Owned(file.into()));
  }

  /// Get the ignoreList field in [SourceMap].
  pub fn ignore_list(&self) -> Option<&[u32]> {
    self.fields.ignore_list.as_deref()
  }

  /// Set the ignoreList field in [SourceMap].
  pub fn set_ignore_list(&mut self, ignore_list: Option<Cow<'a, [u32]>>) {
    self.fields.ignore_list = ignore_list;
  }

  /// Get the decoded mappings in [SourceMap].
  pub fn decoded_mappings(&self) -> impl Iterator<Item = Mapping> + '_ {
    decode_mappings_fields(self.fields())
  }

  /// Get the mappings string in [SourceMap].
  pub fn mappings(&self) -> &str {
    self.fields.mappings.as_ref()
  }

  /// Get the sources field in [SourceMap].
  pub fn sources(&self) -> &[Cow<'_, str>] {
    self.fields.sources.as_ref()
  }

  /// Set the sources field in [SourceMap].
  pub fn set_sources<T, I>(&mut self, sources: I)
  where
    T: Into<String>,
    I: IntoIterator<Item = T>,
  {
    self.fields.sources = Cow::Owned(
      sources
        .into_iter()
        .map(|source| Cow::Owned(source.into()))
        .collect(),
    );
  }

  /// Get the source by index from sources field in [SourceMap].
  pub fn get_source(&self, index: usize) -> Option<&str> {
    self.fields.sources.get(index).map(AsRef::as_ref)
  }

  /// Get the sourcesContent field in [SourceMap].
  pub fn sources_content(&self) -> &[Cow<'_, str>] {
    self.fields.sources_content.as_ref()
  }

  /// Set the sourcesContent field in [SourceMap].
  pub fn set_sources_content(&mut self, sources_content: Vec<Cow<'a, str>>) {
    self.fields.sources_content = Cow::Owned(sources_content);
  }

  /// Get the source content by index from sourcesContent field in [SourceMap].
  pub fn get_source_content(&self, index: usize) -> Option<&Cow<'_, str>> {
    self.fields.sources_content.get(index)
  }

  /// Get the names field in [SourceMap].
  pub fn names(&self) -> &[Cow<'_, str>] {
    self.fields.names.as_ref()
  }

  /// Set the names field in [SourceMap].
  pub fn set_names<T, I>(&mut self, names: I)
  where
    T: Into<String>,
    I: IntoIterator<Item = T>,
  {
    self.fields.names = Cow::Owned(
      names
        .into_iter()
        .map(|name| Cow::Owned(name.into()))
        .collect(),
    );
  }

  /// Get the name by index from names field in [SourceMap].
  pub fn get_name(&self, index: usize) -> Option<&str> {
    self.fields.names.get(index).map(AsRef::as_ref)
  }

  /// Get the source_root field in [SourceMap].
  pub fn source_root(&self) -> Option<&str> {
    self.fields.source_root.as_deref()
  }

  /// Set the source_root field in [SourceMap].
  pub fn set_source_root(&mut self, source_root: Option<Cow<'a, str>>) {
    self.fields.source_root = source_root;
  }

  /// Set the debug_id field in [SourceMap].
  pub fn set_debug_id(&mut self, debug_id: Option<Cow<'a, str>>) {
    self.fields.debug_id = debug_id;
  }

  /// Get the debug_id field in [SourceMap].
  pub fn get_debug_id(&self) -> Option<&str> {
    self.fields.debug_id.as_deref()
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
    if let Some(file) = &self.fields.file {
      size += 9 + file.len(); // "file":"", + content
    }

    // sources array: each element needs quotes + comma + potential escaping
    // ["src/a.js","src/b.js"] = 2 + (len + 3) * count - 1
    let sources_len: usize = self.fields.sources.iter().map(|s| s.len()).sum();
    size += 2 + sources_len + self.fields.sources.len() * 3;

    // sourcesContent array
    if !self.fields.sources_content.is_empty() {
      let content_len: usize = self.fields.sources_content.iter().map(|c| c.len()).sum();
      // Source content often contains special characters that need escaping
      // Estimate 10% escaping overhead for source content
      size += 19 + content_len + (content_len / 10) + self.fields.sources_content.len() * 3;
    }

    // names array
    let names_len: usize = self.fields.names.iter().map(|n| n.len()).sum();
    size += 2 + names_len + self.fields.names.len() * 3;

    // mappings string (usually the largest part)
    // VLQ mappings rarely need escaping, add small overhead
    size += self.fields.mappings.len() + 14; // "mappings":"...",

    // sourceRoot field
    if let Some(source_root) = &self.fields.source_root {
      size += 15 + source_root.len(); // "sourceRoot":"...",
    }

    // debugId field
    if let Some(debug_id) = &self.fields.debug_id {
      size += 12 + debug_id.len(); // "debugId":"...",
    }

    // ignoreList field: [0,1,2] - numbers as strings
    if let Some(ignore_list) = &self.fields.ignore_list {
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

impl Serialize for SourceMap<'_> {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    self.fields.serialize(serializer)
  }
}

impl<'a, 'b> PartialEq<SourceMap<'b>> for SourceMap<'a> {
  fn eq(&self, other: &SourceMap<'b>) -> bool {
    self.fields == other.fields
  }
}

impl Eq for SourceMap<'_> {}

impl std::fmt::Debug for SourceMap<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    let indent = f.width().unwrap_or(0);
    let indent_str = format!("{:indent$}", "", indent = indent);

    write!(
      f,
      "{indent_str}SourceMap::from_json({:?}.to_string()).unwrap()",
      self.to_json()
    )?;

    Ok(())
  }
}

impl Hash for SourceMap<'_> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.fields.hash(state);
  }
}

fn deserialize_source_map_fields<'a>(value: &'a BorrowedValue<'a>) -> Result<SourceMapFields<'a>> {
  let object = value
    .as_object()
    .ok_or_else(|| simd_json::Error::generic(ErrorType::ExpectedMap))?;
  let mappings = required_string_field(object, "mappings")?;

  Ok(SourceMapFields {
    version: 3,
    file: optional_string_field(object, "file")?,
    sources: Cow::Owned(optional_string_array_field(object, "sources")?),
    sources_content: Cow::Owned(optional_string_array_field(object, "sourcesContent")?),
    names: Cow::Owned(optional_string_array_field(object, "names")?),
    mappings,
    source_root: optional_string_field(object, "sourceRoot")?,
    debug_id: optional_string_field(object, "debugId")?,
    ignore_list: optional_u32_array_field(object, "ignoreList")?,
  })
}

fn required_string_field<'a>(
  object: &'a simd_json::borrowed::Object<'a>,
  key: &str,
) -> Result<Cow<'a, str>> {
  object
    .get(key)
    .and_then(BorrowedValue::as_str)
    .map(Cow::Borrowed)
    .ok_or_else(|| simd_json::Error::generic(ErrorType::ExpectedString).into())
}

fn optional_string_field<'a>(
  object: &'a simd_json::borrowed::Object<'a>,
  key: &str,
) -> Result<Option<Cow<'a, str>>> {
  let Some(value) = object.get(key) else {
    return Ok(None);
  };
  if value.is_null() {
    return Ok(None);
  }
  value
    .as_str()
    .map(|value| Some(Cow::Borrowed(value)))
    .ok_or_else(|| simd_json::Error::generic(ErrorType::ExpectedString).into())
}

fn optional_string_array_field<'a>(
  object: &'a simd_json::borrowed::Object<'a>,
  key: &str,
) -> Result<Vec<Cow<'a, str>>> {
  let Some(value) = object.get(key) else {
    return Ok(Vec::new());
  };
  if value.is_null() {
    return Ok(Vec::new());
  }
  let values = value
    .as_array()
    .ok_or_else(|| simd_json::Error::generic(ErrorType::ExpectedArray))?;
  values
    .iter()
    .map(|value| {
      if value.is_null() {
        Ok(Cow::Borrowed(""))
      } else {
        value
          .as_str()
          .map(Cow::Borrowed)
          .ok_or_else(|| simd_json::Error::generic(ErrorType::ExpectedString).into())
      }
    })
    .collect()
}

fn optional_u32_array_field<'a>(
  object: &'a simd_json::borrowed::Object<'a>,
  key: &str,
) -> Result<Option<Cow<'a, [u32]>>> {
  let Some(value) = object.get(key) else {
    return Ok(None);
  };
  if value.is_null() {
    return Ok(None);
  }
  let values = value
    .as_array()
    .ok_or_else(|| simd_json::Error::generic(ErrorType::ExpectedArray))?;
  values
    .iter()
    .map(|value| {
      value
        .as_u32()
        .ok_or_else(|| simd_json::Error::generic(ErrorType::ExpectedUnsigned).into())
    })
    .collect::<Result<Vec<_>>>()
    .map(|v| Some(Cow::Owned(v)))
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
      source_map: SourceMap::from_json("{\"mappings\": \";\"}".to_string()).unwrap(),
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
        source_map: SourceMap::from_json("{\"mappings\": \";\"}".to_string()).unwrap(),
      }),
      SourceMapSource::new(WithoutOriginalOptions {
        value: "c",
        name: "",
        source_map: SourceMap::from_json("{\"mappings\": \";\"}".to_string()).unwrap(),
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
