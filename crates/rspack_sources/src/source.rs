use std::{
  any::{Any, TypeId},
  borrow::Cow,
  fmt,
  hash::{Hash, Hasher},
  io::Read,
  sync::Arc,
};

use dyn_clone::DynClone;
use serde::Serialize;
use simd_json::{BorrowedValue, ErrorType, prelude::*, to_borrowed_value};

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
  fn map(self: &Arc<Self>, object_pool: &ObjectPool, options: &MapOptions) -> Option<SourceMap>
  where
    Self: Sized + 'static,
  {
    let source: BoxSource = self.clone();
    self.as_ref().map_with_source(source, object_pool, options)
  }

  #[doc(hidden)]
  fn map_with_source(
    &self,
    source: BoxSource,
    object_pool: &ObjectPool,
    options: &MapOptions,
  ) -> Option<SourceMap>;

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
  fn map_with_source(
    &self,
    _source: BoxSource,
    object_pool: &ObjectPool,
    options: &MapOptions,
  ) -> Option<SourceMap> {
    self
      .as_ref()
      .map_with_source(self.clone(), object_pool, options)
  }

  #[inline]
  fn to_writer(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    self.as_ref().to_writer(writer)
  }
}

impl dyn Source {
  /// Get the [SourceMap].
  pub fn map(
    self: &Arc<Self>,
    object_pool: &ObjectPool,
    options: &MapOptions,
  ) -> Option<SourceMap> {
    self
      .as_ref()
      .map_with_source(self.clone(), object_pool, options)
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

  /// Convenience wrapper for concrete sources.
  fn map(&self, object_pool: &ObjectPool, options: &MapOptions) -> Option<SourceMap>
  where
    Self: Clone + Sized + Source + 'static,
  {
    Source::map(&Arc::new(self.clone()), object_pool, options)
  }
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

fn is_all_empty(val: &[Cow<'static, str>]) -> bool {
  if val.is_empty() {
    return true;
  }
  val.iter().all(|s| s.is_empty())
}

pub trait SourceMapJsonInput {
  fn into_json_string(self) -> Result<String>;
}

impl SourceMapJsonInput for String {
  fn into_json_string(self) -> Result<String> {
    Ok(self)
  }
}

impl SourceMapJsonInput for &str {
  fn into_json_string(self) -> Result<String> {
    Ok(self.to_owned())
  }
}

impl SourceMapJsonInput for &String {
  fn into_json_string(self) -> Result<String> {
    Ok(self.to_owned())
  }
}

impl SourceMapJsonInput for Vec<u8> {
  fn into_json_string(self) -> Result<String> {
    String::from_utf8(self).map_err(|err| err.utf8_error().into())
  }
}

impl SourceMapJsonInput for &[u8] {
  fn into_json_string(self) -> Result<String> {
    Ok(std::str::from_utf8(self)?.to_owned())
  }
}

enum SourceMapBacking {
  Json(String),
  Source(BoxSource),
}

impl Clone for SourceMapBacking {
  fn clone(&self) -> Self {
    match self {
      Self::Json(json) => Self::Json(json.clone()),
      Self::Source(source) => Self::Source(source.clone()),
    }
  }
}

impl SourceMapBacking {
  fn json_bytes(&self) -> Option<&[u8]> {
    match self {
      Self::Json(json) => Some(json.as_bytes()),
      _ => None,
    }
  }
}

/// The source map created by [Source::map].
#[derive(Serialize)]
pub struct SourceMap {
  version: u8,
  #[serde(skip_serializing_if = "Option::is_none")]
  file: Option<Cow<'static, str>>,
  sources: Vec<Cow<'static, str>>,
  #[serde(rename = "sourcesContent", skip_serializing_if = "is_all_empty")]
  sources_content: Vec<Cow<'static, str>>,
  names: Vec<Cow<'static, str>>,
  mappings: Cow<'static, str>,
  #[serde(rename = "sourceRoot", skip_serializing_if = "Option::is_none")]
  source_root: Option<Cow<'static, str>>,
  #[serde(rename = "debugId", skip_serializing_if = "Option::is_none")]
  debug_id: Option<Cow<'static, str>>,
  #[serde(rename = "ignoreList", skip_serializing_if = "Option::is_none")]
  ignore_list: Option<Vec<u32>>,
  #[serde(skip)]
  backing: Option<SourceMapBacking>,
}

impl Clone for SourceMap {
  fn clone(&self) -> Self {
    let backing = self.backing.clone();
    let old_buffer = self.backing.as_ref().and_then(SourceMapBacking::json_bytes);
    let new_buffer = backing.as_ref().and_then(SourceMapBacking::json_bytes);

    Self {
      version: self.version,
      file: self
        .file
        .as_ref()
        .map(|file| clone_cow(file, old_buffer, new_buffer)),
      sources: self
        .sources
        .iter()
        .map(|source| clone_cow(source, old_buffer, new_buffer))
        .collect(),
      sources_content: self
        .sources_content
        .iter()
        .map(|source_content| clone_cow(source_content, old_buffer, new_buffer))
        .collect(),
      names: self
        .names
        .iter()
        .map(|name| clone_cow(name, old_buffer, new_buffer))
        .collect(),
      mappings: clone_cow(&self.mappings, old_buffer, new_buffer),
      source_root: self
        .source_root
        .as_ref()
        .map(|source_root| clone_cow(source_root, old_buffer, new_buffer)),
      debug_id: self
        .debug_id
        .as_ref()
        .map(|debug_id| clone_cow(debug_id, old_buffer, new_buffer)),
      ignore_list: self.ignore_list.clone(),
      backing,
    }
  }
}

impl PartialEq for SourceMap {
  fn eq(&self, other: &Self) -> bool {
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

impl Eq for SourceMap {}

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

#[inline]
pub(crate) fn cow_to_static<'a>(value: Cow<'a, str>) -> Cow<'static, str> {
  match value {
    Cow::Borrowed(value) => Cow::Borrowed(str_to_static(value)),
    Cow::Owned(value) => Cow::Owned(value),
  }
}

#[inline]
#[allow(unsafe_code)]
pub(crate) fn str_to_static(value: &str) -> &'static str {
  // SAFETY: Callers only use this for strings borrowed from data retained by
  // SourceMap through `backing`.
  unsafe { std::mem::transmute::<&str, &'static str>(value) }
}

#[allow(clippy::ptr_arg)]
fn clone_cow(
  value: &Cow<'static, str>,
  old_buffer: Option<&[u8]>,
  new_buffer: Option<&[u8]>,
) -> Cow<'static, str> {
  match value {
    Cow::Borrowed(value) => rebase_str(value, old_buffer, new_buffer)
      .map(Cow::Borrowed)
      .unwrap_or(Cow::Borrowed(value)),
    Cow::Owned(value) => Cow::Owned(value.clone()),
  }
}

fn rebase_str(
  value: &'static str,
  old_buffer: Option<&[u8]>,
  new_buffer: Option<&[u8]>,
) -> Option<&'static str> {
  let old_buffer = old_buffer?;
  let new_buffer = new_buffer?;
  let value_start = value.as_ptr() as usize;
  let old_start = old_buffer.as_ptr() as usize;
  let old_end = old_start.checked_add(old_buffer.len())?;
  let value_end = value_start.checked_add(value.len())?;
  if value_start < old_start || value_end > old_end {
    return None;
  }

  let offset = value_start - old_start;
  let bytes = &new_buffer[offset..offset + value.len()];
  #[allow(unsafe_code)]
  // SAFETY: `bytes` is the same byte range in a clone of `old_buffer`, and
  // `value` was already a valid UTF-8 string over that range.
  Some(str_to_static(unsafe {
    std::str::from_utf8_unchecked(bytes)
  }))
}

impl SourceMap {
  /// Create a [SourceMap].
  pub fn new(
    mappings: impl Into<Cow<'static, str>>,
    sources: Vec<Cow<'static, str>>,
    sources_content: Vec<Cow<'static, str>>,
    names: Vec<Cow<'static, str>>,
  ) -> Self {
    Self::new_with_source(mappings.into(), sources, sources_content, names, None)
  }

  pub(crate) fn new_with_source(
    mappings: Cow<'static, str>,
    sources: Vec<Cow<'static, str>>,
    sources_content: Vec<Cow<'static, str>>,
    names: Vec<Cow<'static, str>>,
    source: Option<BoxSource>,
  ) -> Self {
    Self {
      version: 3,
      file: None,
      mappings,
      sources,
      sources_content,
      names,
      source_root: None,
      debug_id: None,
      ignore_list: None,
      backing: source.map(SourceMapBacking::Source),
    }
  }

  /// Get the file field in [SourceMap].
  pub fn file(&self) -> Option<&str> {
    self.file.as_deref()
  }

  /// Set the file field in [SourceMap].
  pub fn set_file<T: AsRef<str>>(&mut self, file: Option<T>) {
    self.file = file.map(|file| Cow::Owned(file.as_ref().to_owned()));
  }

  /// Get the ignoreList field in [SourceMap].
  pub fn ignore_list(&self) -> Option<&[u32]> {
    self.ignore_list.as_deref()
  }

  /// Set the ignoreList field in [SourceMap].
  pub fn set_ignore_list<T: Into<Vec<u32>>>(&mut self, ignore_list: Option<T>) {
    self.ignore_list = ignore_list.map(Into::into);
  }

  /// Get the decoded mappings in [SourceMap].
  pub fn decoded_mappings(&self) -> impl Iterator<Item = Mapping> + '_ {
    decode_mappings(self)
  }

  /// Get the mappings string in [SourceMap].
  pub fn mappings(&self) -> &str {
    self.mappings.as_ref()
  }

  /// Get the sources field in [SourceMap].
  pub fn sources(&self) -> &[Cow<'static, str>] {
    &self.sources
  }

  /// Set the sources field in [SourceMap].
  pub fn set_sources<T, I>(&mut self, sources: I)
  where
    T: AsRef<str>,
    I: IntoIterator<Item = T>,
  {
    self.sources = sources
      .into_iter()
      .map(|source| Cow::Owned(source.as_ref().to_owned()))
      .collect();
  }

  /// Get the source by index from sources field in [SourceMap].
  pub fn get_source(&self, index: usize) -> Option<&str> {
    self.sources.get(index).map(AsRef::as_ref)
  }

  /// Get the sourcesContent field in [SourceMap].
  pub fn sources_content(&self) -> &[Cow<'static, str>] {
    &self.sources_content
  }

  /// Set the sourcesContent field in [SourceMap].
  pub fn set_sources_content<T, I>(&mut self, sources_content: I)
  where
    T: AsRef<str>,
    I: IntoIterator<Item = T>,
  {
    self.sources_content = sources_content
      .into_iter()
      .map(|source_content| Cow::Owned(source_content.as_ref().to_owned()))
      .collect();
  }

  /// Get the source content by index from sourcesContent field in [SourceMap].
  pub fn get_source_content(&self, index: usize) -> Option<&Cow<'static, str>> {
    self.sources_content.get(index)
  }

  /// Get the names field in [SourceMap].
  pub fn names(&self) -> &[Cow<'static, str>] {
    &self.names
  }

  /// Set the names field in [SourceMap].
  pub fn set_names<T, I>(&mut self, names: I)
  where
    T: AsRef<str>,
    I: IntoIterator<Item = T>,
  {
    self.names = names
      .into_iter()
      .map(|name| Cow::Owned(name.as_ref().to_owned()))
      .collect();
  }

  /// Get the name by index from names field in [SourceMap].
  pub fn get_name(&self, index: usize) -> Option<&str> {
    self.names.get(index).map(AsRef::as_ref)
  }

  /// Get the source_root field in [SourceMap].
  pub fn source_root(&self) -> Option<&str> {
    self.source_root.as_deref()
  }

  /// Set the source_root field in [SourceMap].
  pub fn set_source_root<T: AsRef<str>>(&mut self, source_root: Option<T>) {
    self.source_root = source_root.map(|source_root| Cow::Owned(source_root.as_ref().to_owned()));
  }

  /// Set the debug_id field in [SourceMap].
  pub fn set_debug_id<T: AsRef<str>>(&mut self, debug_id: Option<T>) {
    self.debug_id = debug_id.map(|debug_id| Cow::Owned(debug_id.as_ref().to_owned()));
  }

  /// Get the debug_id field in [SourceMap].
  pub fn get_debug_id(&self) -> Option<&str> {
    self.debug_id.as_deref()
  }
}

impl SourceMap {
  /// Create a [SourceMap] from json string.
  pub fn from_json<T: SourceMapJsonInput>(s: T) -> Result<Self> {
    Self::from_string(s.into_json_string()?)
  }

  /// Create a [SourceMap] from [&[u8]].
  pub fn from_slice(s: &[u8]) -> Result<Self> {
    Self::from_json(s)
  }

  /// Create a [SourceMap] from reader.
  pub fn from_reader<R: Read>(mut s: R) -> Result<Self> {
    let mut bytes = Vec::new();
    s.read_to_end(&mut bytes)?;
    Self::from_json(bytes)
  }

  fn from_string(mut json: String) -> Result<Self> {
    #[allow(unsafe_code)]
    // SAFETY: simd-json's borrowed parser mutates the JSON string in-place while
    // keeping it valid UTF-8; the crate uses the same pattern for String inputs.
    let value = to_borrowed_value(unsafe { json.as_bytes_mut() })?;
    let map = parse_borrowed_source_map(&value)?;
    drop(value);
    Ok(map.with_backing(SourceMapBacking::Json(json)))
  }

  fn with_backing(mut self, backing: SourceMapBacking) -> Self {
    self.backing = Some(backing);
    self
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

fn parse_borrowed_source_map(value: &BorrowedValue<'_>) -> Result<SourceMap> {
  let object = value
    .as_object()
    .ok_or_else(|| simd_json::Error::generic(ErrorType::ExpectedMap))?;
  let mappings = required_string_field(object, "mappings")?;

  Ok(SourceMap {
    version: 3,
    file: optional_string_field(object, "file")?,
    sources: optional_string_array_field(object, "sources")?,
    sources_content: optional_string_array_field(object, "sourcesContent")?,
    names: optional_string_array_field(object, "names")?,
    mappings,
    source_root: optional_string_field(object, "sourceRoot")?,
    debug_id: optional_string_field(object, "debugId")?,
    ignore_list: optional_u32_array_field(object, "ignoreList")?,
    backing: None,
  })
}

fn required_string_field(
  object: &simd_json::borrowed::Object<'_>,
  key: &str,
) -> Result<Cow<'static, str>> {
  object
    .get(key)
    .and_then(BorrowedValue::as_str)
    .map(|value| Cow::Borrowed(str_to_static(value)))
    .ok_or_else(|| simd_json::Error::generic(ErrorType::ExpectedString).into())
}

fn optional_string_field(
  object: &simd_json::borrowed::Object<'_>,
  key: &str,
) -> Result<Option<Cow<'static, str>>> {
  let Some(value) = object.get(key) else {
    return Ok(None);
  };
  if value.is_null() {
    return Ok(None);
  }
  value
    .as_str()
    .map(|value| Some(Cow::Borrowed(str_to_static(value))))
    .ok_or_else(|| simd_json::Error::generic(ErrorType::ExpectedString).into())
}

fn optional_string_array_field(
  object: &simd_json::borrowed::Object<'_>,
  key: &str,
) -> Result<Vec<Cow<'static, str>>> {
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
          .map(|value| Cow::Borrowed(str_to_static(value)))
          .ok_or_else(|| simd_json::Error::generic(ErrorType::ExpectedString).into())
      }
    })
    .collect()
}

fn optional_u32_array_field(
  object: &simd_json::borrowed::Object<'_>,
  key: &str,
) -> Result<Option<Vec<u32>>> {
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
    .map(Some)
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
  fn from_json_string_clone_keeps_borrowed_fields_alive() {
    let map = SourceMap::from_json(
      r#"{"version":3,"file":"bundle.js","sources":["a.js"],"sourcesContent":["console.log(1);"],"names":["log"],"mappings":"AAAA","sourceRoot":"/src","debugId":"debug","ignoreList":[0]}"#
        .to_string(),
    )
    .unwrap();
    let cloned = map.clone();
    drop(map);

    assert_eq!(cloned.file(), Some("bundle.js"));
    assert_eq!(cloned.get_source(0), Some("a.js"));
    assert_eq!(
      cloned.get_source_content(0).map(AsRef::as_ref),
      Some("console.log(1);")
    );
    assert_eq!(cloned.get_name(0), Some("log"));
    assert_eq!(cloned.mappings(), "AAAA");
    assert_eq!(cloned.source_root(), Some("/src"));
    assert_eq!(cloned.get_debug_id(), Some("debug"));
    assert_eq!(cloned.ignore_list(), Some([0].as_slice()));
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
