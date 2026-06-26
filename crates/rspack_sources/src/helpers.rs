use core::str;
use std::{borrow::Cow, cell::OnceCell};

use rustc_hash::FxHashMap as HashMap;

use crate::{
  MapOptions, SourceMap, SourceMapFields,
  decoder::MappingsDecoder,
  encoder::{MappingsEncoder, create_encoder},
  linear_map::LinearMap,
  object_pool::ObjectPool,
  source::{Mapping, OriginalLocation},
  source_content_lines::SourceContentLines,
  with_utf16::WithUtf16,
};

pub fn get_map<'a>(
  object_pool: &ObjectPool,
  chunks: &dyn Chunks<'a>,
  options: &MapOptions,
) -> Option<SourceMapFields<'a>> {
  let mut sink = SourceMapSink::new(options.columns);
  chunks.stream_with(
    object_pool,
    &MapOptions {
      columns: options.columns,
      final_source: true,
    },
    &mut sink,
  );
  sink.into_map()
}

struct SourceMapSink<'source> {
  mappings_encoder: MappingsEncoder,
  sources: Vec<Cow<'source, str>>,
  sources_content: Vec<Cow<'source, str>>,
  names: Vec<Cow<'source, str>>,
}

impl<'source> SourceMapSink<'source> {
  fn new(columns: bool) -> Self {
    Self {
      mappings_encoder: create_encoder(columns),
      sources: Vec::new(),
      sources_content: Vec::new(),
      names: Vec::new(),
    }
  }

  fn into_map(mut self) -> Option<SourceMapFields<'source>> {
    let mappings = self.mappings_encoder.drain();
    (!mappings.is_empty()).then_some(SourceMapFields {
      version: 3,
      file: None,
      mappings: Cow::Owned(mappings),
      sources: Cow::Owned(self.sources),
      sources_content: Cow::Owned(self.sources_content),
      names: Cow::Owned(self.names),
      source_root: None,
      debug_id: None,
      ignore_list: None,
    })
  }
}

impl<'chunk, 'source> StreamSink<'chunk, 'source> for SourceMapSink<'source> {
  #[inline]
  fn on_chunk(&mut self, _chunk: Option<TextSpan<'chunk>>, mapping: Mapping) {
    self.mappings_encoder.encode(&mapping);
  }

  #[inline]
  fn on_source(
    &mut self,
    source_index: u32,
    source: Cow<'source, str>,
    source_content: Option<&'source str>,
  ) {
    let source_index = source_index as usize;
    if self.sources.len() <= source_index {
      self.sources.resize(source_index + 1, Cow::Borrowed(""));
    }
    self.sources[source_index] = source;
    if let Some(source_content) = source_content {
      if self.sources_content.len() <= source_index {
        self
          .sources_content
          .resize(source_index + 1, Cow::Borrowed(""));
      }
      self.sources_content[source_index] = Cow::Borrowed(source_content);
    }
  }

  #[inline]
  fn on_name(&mut self, name_index: u32, name: Cow<'source, str>) {
    let name_index = name_index as usize;
    if self.names.len() <= name_index {
      self.names.resize(name_index + 1, Cow::Borrowed(""));
    }
    self.names[name_index] = name;
  }
}

/// A trait for processing source code chunks and generating source maps.
///
/// This trait provides the core functionality for streaming through source code chunks
/// while building source map information. It's designed to handle the transformation
/// of source code into mappings that connect generated code positions to original
/// source positions.
pub trait Chunks<'source> {
  /// Streams through source code chunks and generates source map information.
  ///
  /// This method processes the source code in chunks, calling the provided sink
  /// for each chunk, source reference, and name reference encountered. It's the core
  /// method for building source maps during code transformation.
  fn stream_with<'chunk>(
    &'chunk self,
    object_pool: &ObjectPool,
    options: &MapOptions,
    sink: &mut dyn StreamSink<'chunk, 'source>,
  ) -> crate::helpers::GeneratedInfo;
}

/// Receives source-map streaming events.
pub trait StreamSink<'chunk, 'source> {
  /// Emits a generated chunk and its mapping.
  fn on_chunk(&mut self, chunk: Option<TextSpan<'chunk>>, mapping: Mapping);

  /// Emits a source referenced by later mappings.
  fn on_source(&mut self, index: u32, source: Cow<'source, str>, content: Option<&'source str>);

  /// Emits a symbol name referenced by later mappings.
  fn on_name(&mut self, index: u32, name: Cow<'source, str>);
}

/// Fluent streaming builder for callers that prefer closure callbacks.
pub struct StreamBuilder<'chunk, 'source, 'stream> {
  chunks: &'chunk (dyn Chunks<'source> + 'chunk),
  object_pool: &'stream ObjectPool,
  options: &'stream MapOptions,
}

impl<'source> dyn Chunks<'source> + '_ {
  /// Build a closure-based stream invocation.
  #[inline]
  pub fn stream<'chunk, 'stream>(
    &'chunk self,
    object_pool: &'stream ObjectPool,
    options: &'stream MapOptions,
  ) -> StreamBuilder<'chunk, 'source, 'stream> {
    StreamBuilder {
      chunks: self,
      object_pool,
      options,
    }
  }
}

impl<'chunk, 'source, 'stream> StreamBuilder<'chunk, 'source, 'stream> {
  /// Set the source callback.
  #[inline]
  pub fn on_source<OnSource>(
    self,
    on_source: OnSource,
  ) -> StreamBuilderWithSource<'chunk, 'source, 'stream, OnSource>
  where
    OnSource: FnMut(u32, Cow<'source, str>, Option<&'source str>),
  {
    StreamBuilderWithSource {
      chunks: self.chunks,
      object_pool: self.object_pool,
      options: self.options,
      on_source,
    }
  }

  /// Execute with only a chunk callback.
  #[inline]
  pub fn on_chunk<OnChunk>(self, on_chunk: OnChunk) -> GeneratedInfo
  where
    OnChunk: FnMut(Option<TextSpan<'chunk>>, Mapping),
  {
    self
      .on_source(|_, _, _| {})
      .on_name(|_, _| {})
      .on_chunk(on_chunk)
  }
}

/// Fluent streaming builder after registering the source callback.
pub struct StreamBuilderWithSource<'chunk, 'source, 'stream, OnSource> {
  chunks: &'chunk (dyn Chunks<'source> + 'chunk),
  object_pool: &'stream ObjectPool,
  options: &'stream MapOptions,
  on_source: OnSource,
}

impl<'chunk, 'source, 'stream, OnSource> StreamBuilderWithSource<'chunk, 'source, 'stream, OnSource>
where
  OnSource: FnMut(u32, Cow<'source, str>, Option<&'source str>),
{
  /// Set the name callback.
  #[inline]
  pub fn on_name<OnName>(
    self,
    on_name: OnName,
  ) -> StreamBuilderWithSourceAndName<'chunk, 'source, 'stream, OnSource, OnName>
  where
    OnName: FnMut(u32, Cow<'source, str>),
  {
    StreamBuilderWithSourceAndName {
      chunks: self.chunks,
      object_pool: self.object_pool,
      options: self.options,
      on_source: self.on_source,
      on_name,
    }
  }

  /// Execute with source and chunk callbacks.
  #[inline]
  pub fn on_chunk<OnChunk>(self, on_chunk: OnChunk) -> GeneratedInfo
  where
    OnChunk: FnMut(Option<TextSpan<'chunk>>, Mapping),
  {
    self.on_name(|_, _| {}).on_chunk(on_chunk)
  }
}

/// Fluent streaming builder after registering source and name callbacks.
pub struct StreamBuilderWithSourceAndName<'chunk, 'source, 'stream, OnSource, OnName> {
  chunks: &'chunk (dyn Chunks<'source> + 'chunk),
  object_pool: &'stream ObjectPool,
  options: &'stream MapOptions,
  on_source: OnSource,
  on_name: OnName,
}

impl<'chunk, 'source, 'stream, OnSource, OnName>
  StreamBuilderWithSourceAndName<'chunk, 'source, 'stream, OnSource, OnName>
where
  OnSource: FnMut(u32, Cow<'source, str>, Option<&'source str>),
  OnName: FnMut(u32, Cow<'source, str>),
{
  /// Execute the stream with all callbacks.
  #[inline]
  pub fn on_chunk<OnChunk>(self, on_chunk: OnChunk) -> GeneratedInfo
  where
    OnChunk: FnMut(Option<TextSpan<'chunk>>, Mapping),
  {
    let mut sink = ClosureStreamSink {
      on_source: self.on_source,
      on_name: self.on_name,
      on_chunk,
    };
    self
      .chunks
      .stream_with(self.object_pool, self.options, &mut sink)
  }
}

struct ClosureStreamSink<OnSource, OnName, OnChunk> {
  on_source: OnSource,
  on_name: OnName,
  on_chunk: OnChunk,
}

impl<'chunk, 'source, OnSource, OnName, OnChunk> StreamSink<'chunk, 'source>
  for ClosureStreamSink<OnSource, OnName, OnChunk>
where
  OnSource: FnMut(u32, Cow<'source, str>, Option<&'source str>),
  OnName: FnMut(u32, Cow<'source, str>),
  OnChunk: FnMut(Option<TextSpan<'chunk>>, Mapping),
{
  #[inline]
  fn on_chunk(&mut self, chunk: Option<TextSpan<'chunk>>, mapping: Mapping) {
    (self.on_chunk)(chunk, mapping);
  }

  #[inline]
  fn on_source(&mut self, index: u32, source: Cow<'source, str>, content: Option<&'source str>) {
    (self.on_source)(index, source, content);
  }

  #[inline]
  fn on_name(&mut self, index: u32, name: Cow<'source, str>) {
    (self.on_name)(index, name);
  }
}

/// [StreamChunks] abstraction, see [webpack-sources source.streamChunks](https://github.com/webpack/webpack-sources/blob/9f98066311d53a153fdc7c633422a1d086528027/lib/helpers/streamChunks.js#L13).
pub trait StreamChunks {
  /// [StreamChunks] abstraction
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AsciiHit {
  Ascii,
  NotAscii,
  Unknown,
}

impl AsciiHit {
  #[inline]
  fn from_is_ascii(is_ascii: bool) -> Self {
    if is_ascii {
      Self::Ascii
    } else {
      Self::NotAscii
    }
  }

  #[inline]
  fn for_subspan(self) -> Self {
    if matches!(self, Self::Ascii) {
      Self::Ascii
    } else {
      Self::Unknown
    }
  }
}

/// A borrowed text span with ASCII metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextSpan<'a> {
  text: &'a str,
  ascii_hit: AsciiHit,
}

impl<'a> TextSpan<'a> {
  /// Create a text span without computing its ASCII status.
  #[inline]
  pub fn new(text: &'a str) -> Self {
    Self {
      text,
      ascii_hit: AsciiHit::Unknown,
    }
  }

  /// Create a text span from an ASCII fast-path hint.
  #[inline]
  pub fn with_ascii(text: &'a str, is_ascii: bool) -> Self {
    debug_assert!(!is_ascii || text.is_ascii());
    Self {
      text,
      ascii_hit: AsciiHit::from_is_ascii(is_ascii),
    }
  }

  /// Create a text span from known ASCII status.
  #[inline]
  pub(crate) fn with_known(text: &'a str, is_ascii: bool) -> Self {
    debug_assert_eq!(is_ascii, text.is_ascii());
    Self {
      text,
      ascii_hit: AsciiHit::from_is_ascii(is_ascii),
    }
  }

  /// Return the span text.
  #[inline]
  pub fn as_str(&self) -> &'a str {
    self.text
  }

  /// Return the byte length of the span text.
  #[inline]
  pub fn len(&self) -> usize {
    self.text.len()
  }

  /// Return whether the span text is empty.
  #[inline]
  pub fn is_empty(&self) -> bool {
    self.text.is_empty()
  }

  /// Return whether the span text ends with a character.
  #[inline]
  pub fn ends_with(&self, ch: char) -> bool {
    self.text.ends_with(ch)
  }

  /// Return whether this span is ASCII.
  #[inline]
  pub fn is_ascii(&self) -> bool {
    match self.ascii_hit {
      AsciiHit::Ascii => true,
      AsciiHit::NotAscii => false,
      AsciiHit::Unknown => self.text.is_ascii(),
    }
  }

  /// Return the UTF-16 length of the span.
  #[inline]
  pub fn utf16_len(&self) -> usize {
    self.utf16_len_of(self.text)
  }

  #[inline]
  pub(crate) fn subspan(&self, text: &'a str) -> Self {
    Self {
      text,
      ascii_hit: self.ascii_hit.for_subspan(),
    }
  }

  #[inline]
  pub(crate) fn utf16_len_of(&self, text: &str) -> usize {
    match self.ascii_hit {
      AsciiHit::Ascii => text.len(),
      AsciiHit::NotAscii => utf16_len(text),
      AsciiHit::Unknown => {
        if text.is_ascii() {
          text.len()
        } else {
          utf16_len(text)
        }
      }
    }
  }

  #[inline]
  pub(crate) fn is_known_ascii(&self) -> bool {
    matches!(self.ascii_hit, AsciiHit::Ascii)
  }

  /// Slice this span by byte offsets.
  #[inline]
  pub fn slice(&self, start: usize, end: usize) -> Self {
    Self {
      text: &self.text[start..end],
      ascii_hit: self.ascii_hit.for_subspan(),
    }
  }

  /// Slice this span from the start to a byte offset.
  #[inline]
  pub fn slice_to(&self, end: usize) -> Self {
    Self {
      text: &self.text[..end],
      ascii_hit: self.ascii_hit.for_subspan(),
    }
  }

  /// Slice this span from a byte offset to the end.
  #[inline]
  pub fn slice_from(&self, start: usize) -> Self {
    Self {
      text: &self.text[start..],
      ascii_hit: self.ascii_hit.for_subspan(),
    }
  }
}

impl AsRef<str> for TextSpan<'_> {
  #[inline]
  fn as_ref(&self) -> &str {
    self.text
  }
}

impl std::ops::Deref for TextSpan<'_> {
  type Target = str;

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.text
  }
}

/// Default stream chunks behavior impl, see [webpack-sources streamChunks](https://github.com/webpack/webpack-sources/blob/9f98066311d53a153fdc7c633422a1d086528027/lib/helpers/streamChunks.js#L15-L35).
pub fn stream_chunks_default<'chunk, 'source, 'map>(
  options: &MapOptions,
  object_pool: &ObjectPool,
  source: &'chunk str,
  source_map: Option<&'source SourceMap<'map>>,
  sink: &mut dyn StreamSink<'chunk, 'source>,
) -> GeneratedInfo
where
  'map: 'source,
{
  stream_chunks_default_fields(
    options,
    object_pool,
    source,
    source_map.map(SourceMap::fields),
    sink,
  )
}

pub(crate) fn stream_chunks_default_fields<'chunk, 'source>(
  options: &MapOptions,
  object_pool: &ObjectPool,
  source: &'chunk str,
  source_map: Option<&'source SourceMapFields<'_>>,
  sink: &mut dyn StreamSink<'chunk, 'source>,
) -> GeneratedInfo {
  let source = TextSpan::new(source);
  if let Some(map) = source_map {
    stream_chunks_of_source_map(options, object_pool, source, map, sink)
  } else {
    stream_chunks_of_raw_source(source, options, sink)
  }
}

/// `GeneratedSourceInfo` abstraction, see [webpack-sources GeneratedSourceInfo](https://github.com/webpack/webpack-sources/blob/9f98066311d53a153fdc7c633422a1d086528027/lib/helpers/getGeneratedSourceInfo.js)
#[derive(Debug, PartialEq, Eq)]
pub struct GeneratedInfo {
  /// Generated line
  pub generated_line: u32,
  /// Generated column
  pub generated_column: u32,
}

/// Decodes the given mappings string into an iterator of `Mapping` items.
pub fn decode_mappings<'a>(source_map: &'a SourceMap<'_>) -> impl Iterator<Item = Mapping> + 'a {
  decode_mappings_fields(source_map.fields())
}

pub(crate) fn decode_mappings_fields<'a>(
  source_map: &'a SourceMapFields<'_>,
) -> impl Iterator<Item = Mapping> + 'a {
  MappingsDecoder::new(source_map.mappings())
}

/// Encodes the given iterator of `Mapping` items into a `String`.
pub fn encode_mappings(mappings: impl Iterator<Item = Mapping>) -> String {
  let mut encoder = create_encoder(true);
  mappings.for_each(|mapping| encoder.encode(&mapping));
  encoder.drain()
}

/// Compute the number of UTF-16 code units for a UTF-8 string, using SIMD.
///
/// Formula: `utf16_len = byte_length - continuation_bytes + four_byte_leaders`
#[inline]
pub fn utf16_len(s: &str) -> usize {
  simd_utf16_len::utf16_len(s)
}

pub struct PotentialTokens<'a> {
  text: &'a str,
}

impl<'a> Iterator for PotentialTokens<'a> {
  type Item = &'a str;

  #[allow(unsafe_code)]
  fn next(&mut self) -> Option<Self::Item> {
    if self.text.is_empty() {
      return None;
    }

    let bytes = self.text.as_bytes();
    let mut split_idx = bytes.len();

    let primary = memchr::memchr3(b'\n', b';', b'{', bytes);
    let limit = primary.unwrap_or(bytes.len());
    let closing_brace = memchr::memchr(b'}', &bytes[..limit]);

    if let Some(boundary) = closing_brace.or(primary) {
      split_idx = boundary;

      for &b in &bytes[boundary..] {
        match b {
          b';' | b' ' | b'{' | b'}' | b'\r' | b'\t' => split_idx += 1,
          b'\n' => {
            split_idx += 1;
            break;
          }
          _ => break,
        }
      }
    }

    let text = unsafe { self.text.get_unchecked(..split_idx) };
    self.text = unsafe { self.text.get_unchecked(split_idx..) };

    Some(text)
  }
}

// /[^\n;{}]+[;{} \r\t]*\n?|[;{} \r\t]+\n?|\n/g
pub fn split_into_potential_tokens<'a>(text: &'a str) -> PotentialTokens<'a> {
  PotentialTokens { text }
}

/// Split the string with a needle, each string will contain the needle.
///
/// Copied and modified from https://github.com/rust-lang/cargo/blob/30efe860c0e4adc1a6d7057ad223dc6e47d34edf/src/cargo/sources/registry/index.rs#L1048-L1072
fn split(haystack: &str, needle: u8) -> impl Iterator<Item = &str> {
  struct Split<'a> {
    haystack: &'a str,
    needle: u8,
  }

  impl<'a> Iterator for Split<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
      if self.haystack.is_empty() {
        return None;
      }
      let (ret, remaining) = match memchr::memchr(self.needle, self.haystack.as_bytes()) {
        Some(pos) => (&self.haystack[..=pos], &self.haystack[pos + 1..]),
        None => (self.haystack, ""),
      };
      self.haystack = remaining;
      Some(ret)
    }
  }

  Split { haystack, needle }
}

// /[^\n]+\n?|\n/g
pub fn split_into_lines(source: &str) -> impl Iterator<Item = &str> {
  split(source, b'\n')
}

pub(crate) fn get_generated_source_info(source: TextSpan<'_>) -> GeneratedInfo {
  let (generated_line, generated_column) = if source.ends_with('\n') {
    (split_into_lines(source.as_str()).count() + 1, 0)
  } else {
    let mut line_count = 0;
    let mut last_line = "";

    for line in split_into_lines(source.as_str()) {
      line_count += 1;
      last_line = line;
    }

    (line_count.max(1), source.utf16_len_of(last_line))
  };
  GeneratedInfo {
    generated_line: generated_line as u32,
    generated_column: generated_column as u32,
  }
}

pub fn stream_chunks_of_raw_source<'chunk, 'source>(
  source: TextSpan<'chunk>,
  options: &MapOptions,
  sink: &mut dyn StreamSink<'chunk, 'source>,
) -> GeneratedInfo {
  if options.final_source {
    return get_generated_source_info(source);
  }

  let mut line = 1;
  let mut last_line = None;
  for l in split_into_lines(source.as_str()) {
    sink.on_chunk(
      Some(source.subspan(l)),
      Mapping {
        generated_line: line,
        generated_column: 0,
        original: None,
      },
    );
    line += 1;
    last_line = Some(l);
  }
  if let Some(last_line) = last_line.filter(|last_line| !last_line.ends_with('\n')) {
    GeneratedInfo {
      generated_line: line - 1,
      generated_column: last_line.len() as u32,
    }
  } else {
    GeneratedInfo {
      generated_line: line,
      generated_column: 0,
    }
  }
}

pub fn stream_chunks_of_source_map<'chunk, 'source>(
  options: &MapOptions,
  object_pool: &ObjectPool,
  source: TextSpan<'chunk>,
  source_map: &'source SourceMapFields<'_>,
  sink: &mut dyn StreamSink<'chunk, 'source>,
) -> GeneratedInfo {
  match options {
    MapOptions {
      columns: true,
      final_source: true,
      ..
    } => stream_chunks_of_source_map_final(source, source_map, sink),
    MapOptions {
      columns: true,
      final_source: false,
      ..
    } => stream_chunks_of_source_map_full(object_pool, source, source_map, sink),
    MapOptions {
      columns: false,
      final_source: true,
      ..
    } => stream_chunks_of_source_map_lines_final(source, source_map, sink),
    MapOptions {
      columns: false,
      final_source: false,
      ..
    } => stream_chunks_of_source_map_lines_full(source, source_map, sink),
  }
}

fn get_source<'a>(source_map: &SourceMapFields, source: &'a str) -> Cow<'a, str> {
  let source_root = source_map.source_root();
  match source_root {
    Some("") => Cow::Borrowed(source),
    Some(root) if root.ends_with('/') => Cow::Owned(format!("{root}{source}")),
    Some(root) => Cow::Owned(format!("{root}/{source}")),
    None => Cow::Borrowed(source),
  }
}

fn stream_chunks_of_source_map_final<'chunk, 'source>(
  source: TextSpan<'chunk>,
  source_map: &'source SourceMapFields<'_>,
  sink: &mut dyn StreamSink<'chunk, 'source>,
) -> GeneratedInfo {
  let result = get_generated_source_info(source);
  if result.generated_line == 1 && result.generated_column == 0 {
    return result;
  }
  for (i, source) in source_map.sources().iter().enumerate() {
    sink.on_source(
      i as u32,
      get_source(source_map, source),
      source_map.get_source_content(i).map(AsRef::as_ref),
    )
  }
  for (i, name) in source_map.names().iter().enumerate() {
    sink.on_name(i as u32, Cow::Borrowed(name));
  }
  let mut mapping_active_line = 0;
  let mut on_mapping = |mapping: Mapping| {
    if mapping.generated_line >= result.generated_line
      && (mapping.generated_column >= result.generated_column
        || mapping.generated_line > result.generated_line)
    {
      return;
    }
    if let Some(original) = mapping.original {
      sink.on_chunk(
        None,
        Mapping {
          generated_line: mapping.generated_line,
          generated_column: mapping.generated_column,
          original: Some(original),
        },
      );
      mapping_active_line = mapping.generated_line;
    } else if mapping_active_line == mapping.generated_line {
      sink.on_chunk(
        None,
        Mapping {
          generated_line: mapping.generated_line,
          generated_column: mapping.generated_column,
          original: None,
        },
      );
    }
  };
  for mapping in source_map.decoded_mappings() {
    on_mapping(mapping);
  }
  result
}

fn stream_chunks_of_source_map_full<'chunk, 'source, 'object_pool>(
  object_pool: &'object_pool ObjectPool,
  source: TextSpan<'chunk>,
  source_map: &'source SourceMapFields<'_>,
  sink: &mut dyn StreamSink<'chunk, 'source>,
) -> GeneratedInfo {
  let lines = split_into_lines(source.as_str())
    .map(|line| WithUtf16::with_known(object_pool, line, source.is_known_ascii()))
    .collect::<Vec<WithUtf16<'object_pool, 'chunk>>>();

  if lines.is_empty() {
    return GeneratedInfo {
      generated_line: 1,
      generated_column: 0,
    };
  }
  for (i, source) in source_map.sources().iter().enumerate() {
    sink.on_source(
      i as u32,
      get_source(source_map, source),
      source_map.get_source_content(i).map(AsRef::as_ref),
    )
  }
  for (i, name) in source_map.names().iter().enumerate() {
    sink.on_name(i as u32, Cow::Borrowed(name));
  }
  let last_line = &lines[lines.len() - 1].line;
  let last_new_line = last_line.ends_with('\n');
  let final_line: u32 = if last_new_line {
    lines.len() + 1
  } else {
    lines.len()
  } as u32;
  let final_column: u32 = if last_new_line {
    0
  } else {
    source.utf16_len_of(last_line)
  } as u32;
  let mut current_generated_line: u32 = 1;
  let mut current_generated_column: u32 = 0;
  let mut mapping_active = false;
  let mut active_mapping_original: Option<OriginalLocation> = None;

  let mut on_mapping = |mapping: Mapping| {
    if mapping_active && current_generated_line as usize <= lines.len() {
      let chunk: &str;
      let mapping_line = current_generated_line;
      let mapping_column = current_generated_column;
      let line = &lines[(current_generated_line - 1) as usize];
      if mapping.generated_line != current_generated_line {
        chunk = line.substring(current_generated_column as usize, usize::MAX);
        current_generated_line += 1;
        current_generated_column = 0;
      } else {
        chunk = line.substring(
          current_generated_column as usize,
          mapping.generated_column as usize,
        );
        current_generated_column = mapping.generated_column;
      }
      if !chunk.is_empty() {
        sink.on_chunk(
          Some(source.subspan(chunk)),
          Mapping {
            generated_line: mapping_line,
            generated_column: mapping_column,
            original: active_mapping_original,
          },
        )
      }
      mapping_active = false;
    }
    if mapping.generated_line > current_generated_line && current_generated_column > 0 {
      if current_generated_line as usize <= lines.len() {
        let chunk = lines[(current_generated_line - 1) as usize]
          .substring(current_generated_column as usize, usize::MAX);
        sink.on_chunk(
          Some(source.subspan(chunk)),
          Mapping {
            generated_line: current_generated_line,
            generated_column: current_generated_column,
            original: None,
          },
        );
      }
      current_generated_line += 1;
      current_generated_column = 0;
    }
    while mapping.generated_line > current_generated_line {
      if current_generated_line as usize <= lines.len() {
        let chunk = &lines[(current_generated_line as usize) - 1].line;
        sink.on_chunk(
          Some(source.subspan(chunk)),
          Mapping {
            generated_line: current_generated_line,
            generated_column: 0,
            original: None,
          },
        );
      }
      current_generated_line += 1;
    }
    if mapping.generated_column > current_generated_column {
      if current_generated_line as usize <= lines.len() {
        let chunk = lines[(current_generated_line as usize) - 1].substring(
          current_generated_column as usize,
          mapping.generated_column as usize,
        );
        sink.on_chunk(
          Some(source.subspan(chunk)),
          Mapping {
            generated_line: current_generated_line,
            generated_column: current_generated_column,
            original: None,
          },
        )
      }
      current_generated_column = mapping.generated_column;
    }
    if let Some(original) = mapping.original.filter(|_| {
      mapping.generated_line < final_line
        || (mapping.generated_line == final_line && mapping.generated_column < final_column)
    }) {
      mapping_active = true;
      active_mapping_original = Some(original);
    }
  };

  for mapping in source_map.decoded_mappings() {
    on_mapping(mapping);
  }
  on_mapping(Mapping {
    generated_line: final_line,
    generated_column: final_column,
    original: None,
  });
  GeneratedInfo {
    generated_line: final_line,
    generated_column: final_column,
  }
}

fn stream_chunks_of_source_map_lines_final<'chunk, 'source>(
  source: TextSpan<'chunk>,
  source_map: &'source SourceMapFields<'_>,
  sink: &mut dyn StreamSink<'chunk, 'source>,
) -> GeneratedInfo {
  let result = get_generated_source_info(source);
  if result.generated_line == 1 && result.generated_column == 0 {
    return GeneratedInfo {
      generated_line: 1,
      generated_column: 0,
    };
  }
  for (i, source) in source_map.sources().iter().enumerate() {
    sink.on_source(
      i as u32,
      get_source(source_map, source),
      source_map.get_source_content(i).map(AsRef::as_ref),
    )
  }
  let final_line = if result.generated_column == 0 {
    result.generated_line - 1
  } else {
    result.generated_line
  };
  let mut current_generated_line = 1;

  let mut on_mapping = |mut mapping: Mapping| {
    if let Some(original) = mapping.original.as_mut().filter(|_| {
      current_generated_line <= mapping.generated_line && mapping.generated_line <= final_line
    }) {
      mapping.generated_column = 0;
      original.name_index = None;
      current_generated_line = mapping.generated_line + 1;
      sink.on_chunk(None, mapping);
    }
  };
  for mapping in source_map.decoded_mappings() {
    on_mapping(mapping);
  }
  result
}

fn stream_chunks_of_source_map_lines_full<'chunk, 'source>(
  source: TextSpan<'chunk>,
  source_map: &'source SourceMapFields<'_>,
  sink: &mut dyn StreamSink<'chunk, 'source>,
) -> GeneratedInfo {
  let lines: Vec<&str> = split_into_lines(source.as_str()).collect();
  if lines.is_empty() {
    return GeneratedInfo {
      generated_line: 1,
      generated_column: 0,
    };
  }
  for (i, source) in source_map.sources().iter().enumerate() {
    sink.on_source(
      i as u32,
      get_source(source_map, source),
      source_map.get_source_content(i).map(AsRef::as_ref),
    )
  }
  let mut current_generated_line = 1;
  let mut on_mapping = |mut mapping: Mapping| {
    if mapping.original.is_none()
      || mapping.generated_line < current_generated_line
      || mapping.generated_line as usize > lines.len()
    {
      return;
    }
    while mapping.generated_line > current_generated_line {
      if current_generated_line as usize <= lines.len() {
        let chunk = &lines[current_generated_line as usize - 1];
        sink.on_chunk(
          Some(source.subspan(chunk)),
          Mapping {
            generated_line: current_generated_line,
            generated_column: 0,
            original: None,
          },
        );
      }
      current_generated_line += 1;
    }
    if let Some(original) = mapping
      .original
      .as_mut()
      .filter(|_| mapping.generated_line as usize <= lines.len())
    {
      let chunk = &lines[current_generated_line as usize - 1];
      mapping.generated_column = 0;
      original.name_index = None;
      sink.on_chunk(Some(source.subspan(chunk)), mapping);
      current_generated_line += 1;
    }
  };
  for mapping in source_map.decoded_mappings() {
    on_mapping(mapping);
  }
  while current_generated_line as usize <= lines.len() {
    let chunk = &lines[current_generated_line as usize - 1];
    sink.on_chunk(
      Some(source.subspan(chunk)),
      Mapping {
        generated_line: current_generated_line,
        generated_column: 0,
        original: None,
      },
    );
    current_generated_line += 1;
  }
  let last_line = &lines[lines.len() - 1];
  let last_new_line = last_line.ends_with('\n');
  let final_line = if last_new_line {
    lines.len() + 1
  } else {
    lines.len()
  } as u32;
  let final_column = if last_new_line {
    0
  } else {
    source.utf16_len_of(last_line)
  } as u32;
  GeneratedInfo {
    generated_line: final_line,
    generated_column: final_column,
  }
}

#[derive(Debug)]
struct SourceMapLineData<'a> {
  pub mappings_data: Vec<i64>,
  pub chunks: Vec<TextSpan<'a>>,
}

type InnerSourceIndexValueMapping<'a> = LinearMap<(Cow<'a, str>, Option<&'a str>)>;

#[allow(clippy::too_many_arguments)]
pub fn stream_chunks_of_combined_source_map<'chunk, 'source, 'map, 'inner_map, 'object_pool>(
  options: &MapOptions,
  object_pool: &'object_pool ObjectPool,
  source: &'chunk str,
  source_map: &'source SourceMapFields<'map>,
  inner_source_name: &'source str,
  inner_source: Option<&'source str>,
  inner_source_map: &'source SourceMapFields<'inner_map>,
  remove_inner_source: bool,
  sink: &mut dyn StreamSink<'chunk, 'source>,
) -> GeneratedInfo
where
  'map: 'source,
  'inner_map: 'source,
{
  let mut sink = CombinedSourceMapSink {
    options,
    object_pool,
    inner_source_name,
    inner_source,
    inner_source_map,
    remove_inner_source,
    sink,
    source_mapping: HashMap::default(),
    name_mapping: HashMap::default(),
    source_index_mapping: LinearMap::default(),
    name_index_mapping: LinearMap::default(),
    name_index_value_mapping: LinearMap::default(),
    inner_source_index: -2,
    inner_source_index_mapping: LinearMap::default(),
    inner_source_index_value_mapping: LinearMap::default(),
    inner_source_contents: LinearMap::default(),
    inner_source_content_lines: LinearMap::default(),
    inner_name_index_mapping: LinearMap::default(),
    inner_name_index_value_mapping: LinearMap::default(),
    inner_source_map_line_data: Vec::new(),
  };
  stream_chunks_of_source_map(
    options,
    object_pool,
    TextSpan::new(source),
    source_map,
    &mut sink,
  )
}

struct CombinedSourceMapSink<'sink, 'chunk, 'source, 'inner_map, 'object_pool> {
  options: &'sink MapOptions,
  object_pool: &'object_pool ObjectPool,
  inner_source_name: &'source str,
  inner_source: Option<&'source str>,
  inner_source_map: &'source SourceMapFields<'inner_map>,
  remove_inner_source: bool,
  sink: &'sink mut dyn StreamSink<'chunk, 'source>,
  source_mapping: HashMap<Cow<'source, str>, u32>,
  name_mapping: HashMap<Cow<'source, str>, u32>,
  source_index_mapping: LinearMap<i64>,
  name_index_mapping: LinearMap<i64>,
  name_index_value_mapping: LinearMap<Cow<'source, str>>,
  inner_source_index: i64,
  inner_source_index_mapping: LinearMap<i64>,
  inner_source_index_value_mapping: InnerSourceIndexValueMapping<'source>,
  inner_source_contents: LinearMap<Option<Cow<'source, str>>>,
  inner_source_content_lines:
    LinearMap<OnceCell<Option<SourceContentLines<'object_pool, 'source>>>>,
  inner_name_index_mapping: LinearMap<i64>,
  inner_name_index_value_mapping: LinearMap<Cow<'source, str>>,
  inner_source_map_line_data: Vec<SourceMapLineData<'source>>,
}

impl<'sink, 'chunk, 'source, 'inner_map, 'object_pool>
  CombinedSourceMapSink<'sink, 'chunk, 'source, 'inner_map, 'object_pool>
where
  'inner_map: 'source,
{
  fn find_inner_mapping(&self, line: i64, column: i64) -> Option<u32> {
    if line <= 0 || line as usize > self.inner_source_map_line_data.len() {
      return None;
    }
    let mappings_data = &self.inner_source_map_line_data[line as usize - 1].mappings_data;
    let mut l = 0;
    let mut r = mappings_data.len() / 5;
    while l < r {
      let m = (l + r) >> 1;
      if mappings_data[m * 5] <= column {
        l = m + 1;
      } else {
        r = m;
      }
    }
    if l == 0 {
      return None;
    }
    Some(l as u32 - 1)
  }

  fn emit_source(
    &mut self,
    source: Cow<'source, str>,
    source_content: Option<&'source str>,
  ) -> u32 {
    if let Some(index) = self.source_mapping.get(&source).copied() {
      return index;
    }
    let index = self.source_mapping.len() as u32;
    self.source_mapping.insert(source.clone(), index);
    self.sink.on_source(index, source, source_content);
    index
  }

  fn emit_name(&mut self, name: Cow<'source, str>) -> u32 {
    if let Some(index) = self.name_mapping.get(&name).copied() {
      return index;
    }
    let index = self.name_mapping.len() as u32;
    self.name_mapping.insert(name.clone(), index);
    self.sink.on_name(index, name);
    index
  }

  fn source_content_lines(
    &self,
    source_index: u32,
  ) -> Option<&SourceContentLines<'object_pool, 'source>> {
    self
      .inner_source_content_lines
      .get(&source_index)
      .and_then(|once_cell| {
        once_cell
          .get_or_init(|| {
            self
              .inner_source_contents
              .get(&source_index)
              .and_then(|source_content| {
                source_content
                  .clone()
                  .map(|source_content| SourceContentLines::new(self.object_pool, source_content))
              })
          })
          .as_ref()
      })
  }

  fn record_inner_source_map(&mut self, source_content: &'source str) {
    let mut sink = InnerSourceMapSink {
      inner_source_map_line_data: &mut self.inner_source_map_line_data,
      inner_source_contents: &mut self.inner_source_contents,
      inner_source_content_lines: &mut self.inner_source_content_lines,
      inner_source_index_mapping: &mut self.inner_source_index_mapping,
      inner_source_index_value_mapping: &mut self.inner_source_index_value_mapping,
      inner_name_index_mapping: &mut self.inner_name_index_mapping,
      inner_name_index_value_mapping: &mut self.inner_name_index_value_mapping,
    };
    stream_chunks_of_source_map(
      &MapOptions {
        columns: self.options.columns,
        final_source: false,
      },
      self.object_pool,
      TextSpan::new(source_content),
      self.inner_source_map,
      &mut sink,
    );
  }

  fn emit_inner_mapping(
    &mut self,
    chunk: Option<TextSpan<'chunk>>,
    mapping: Mapping,
    name_index: i64,
    original_line: i64,
    original_column: i64,
  ) -> bool {
    let Some(idx) = self.find_inner_mapping(original_line, original_column) else {
      return false;
    };
    let idx = idx as usize;
    let (
      inner_source_index,
      inner_original_line,
      mut inner_original_column,
      mut inner_name_index,
      inner_generated_column,
      inner_chunk,
    ) = {
      let data = &self.inner_source_map_line_data[original_line as usize - 1];
      let mappings_data = &data.mappings_data;
      let mi = idx * 5;
      (
        mappings_data[mi + 1],
        mappings_data[mi + 2],
        mappings_data[mi + 3],
        mappings_data[mi + 4],
        mappings_data[mi],
        data.chunks[idx],
      )
    };
    if inner_source_index < 0 {
      return false;
    }

    let inner_source_index = inner_source_index as u32;
    let location_in_chunk = original_column - inner_generated_column;
    if location_in_chunk > 0
      && let Some(original_source_lines) = self.source_content_lines(inner_source_index)
    {
      let original_chunk = original_source_lines
        .get(inner_original_line as usize - 1)
        .map(|lines| {
          let start = inner_original_column as usize;
          let end = start + location_in_chunk as usize;
          lines.substring(start, end)
        });
      if let Some(original_chunk) = original_chunk
        && original_chunk.len() <= inner_chunk.len()
        && inner_chunk
          .as_str()
          .get(..original_chunk.len())
          .is_some_and(|slice| slice == original_chunk)
      {
        inner_original_column += location_in_chunk;
        inner_name_index = -1;
      }
    }

    let mut source_index = self
      .inner_source_index_mapping
      .get(&inner_source_index)
      .copied()
      .unwrap_or(-2);
    if source_index == -2 {
      let (source, source_content) = self
        .inner_source_index_value_mapping
        .get(&inner_source_index)
        .cloned()
        .unwrap_or(("".into(), None));
      source_index = self.emit_source(source, source_content) as i64;
      self
        .inner_source_index_mapping
        .insert(inner_source_index, source_index);
    }

    let final_name_index = self.resolve_inner_name_index(
      inner_name_index,
      name_index,
      inner_source_index,
      inner_original_line,
      inner_original_column,
    );

    self.sink.on_chunk(
      chunk,
      Mapping {
        generated_line: mapping.generated_line,
        generated_column: mapping.generated_column,
        original: (source_index >= 0).then_some(OriginalLocation {
          source_index: source_index as u32,
          original_line: inner_original_line as u32,
          original_column: inner_original_column as u32,
          name_index: (final_name_index >= 0).then_some(final_name_index as u32),
        }),
      },
    );
    true
  }

  fn resolve_inner_name_index(
    &mut self,
    inner_name_index: i64,
    name_index: i64,
    inner_source_index: u32,
    inner_original_line: i64,
    inner_original_column: i64,
  ) -> i64 {
    if inner_name_index >= 0 {
      let inner_name_index = inner_name_index as u32;
      let mut final_name_index = self
        .inner_name_index_mapping
        .get(&inner_name_index)
        .copied()
        .unwrap_or(-2);
      if final_name_index == -2 {
        final_name_index = self
          .inner_name_index_value_mapping
          .get(&inner_name_index)
          .cloned()
          .map_or(-1, |name| self.emit_name(name) as i64);
        self
          .inner_name_index_mapping
          .insert(inner_name_index, final_name_index);
      }
      return final_name_index;
    }

    if name_index < 0 {
      return -1;
    }

    let name_index = name_index as u32;
    let Some(name) = self.name_index_value_mapping.get(&name_index).cloned() else {
      return -1;
    };
    let Some(original_source_lines) = self.source_content_lines(inner_source_index) else {
      return -1;
    };
    let original_name = original_source_lines
      .get(inner_original_line as usize - 1)
      .map_or("", |line| {
        let start = inner_original_column as usize;
        let end = start + name.len();
        line.substring(start, end)
      });
    if name != original_name {
      return -1;
    }

    let mut final_name_index = self
      .name_index_mapping
      .get(&name_index)
      .copied()
      .unwrap_or(-2);
    if final_name_index == -2 {
      final_name_index = self.emit_name(name) as i64;
      self.name_index_mapping.insert(name_index, final_name_index);
    }
    final_name_index
  }
}

impl<'sink, 'chunk, 'source, 'inner_map, 'object_pool> StreamSink<'chunk, 'source>
  for CombinedSourceMapSink<'sink, 'chunk, 'source, 'inner_map, 'object_pool>
where
  'inner_map: 'source,
{
  fn on_chunk(&mut self, chunk: Option<TextSpan<'chunk>>, mapping: Mapping) {
    let source_index = mapping
      .original
      .as_ref()
      .map_or(-1, |o| o.source_index as i64);
    let original_line = mapping
      .original
      .as_ref()
      .map_or(-1, |o| o.original_line as i64);
    let original_column = mapping
      .original
      .as_ref()
      .map_or(-1, |o| o.original_column as i64);
    let name_index = mapping
      .original
      .as_ref()
      .and_then(|o| o.name_index)
      .map(|i| i as i64)
      .unwrap_or(-1);

    if source_index == self.inner_source_index {
      let source_index = source_index as u32;
      if self.emit_inner_mapping(chunk, mapping, name_index, original_line, original_column) {
        return;
      }

      if self.remove_inner_source {
        self.sink.on_chunk(
          chunk,
          Mapping {
            generated_line: mapping.generated_line,
            generated_column: mapping.generated_column,
            original: None,
          },
        );
        return;
      }

      if self.source_index_mapping.get(&source_index) == Some(&-2) {
        let global_index =
          self.emit_source(Cow::Borrowed(self.inner_source_name), self.inner_source);
        self
          .source_index_mapping
          .insert(source_index, global_index as i64);
      }
    }

    let final_source_index = if source_index < 0 {
      -1
    } else {
      let source_index = source_index as u32;
      self
        .source_index_mapping
        .get(&source_index)
        .copied()
        .unwrap_or(-1)
    };
    if final_source_index < 0 {
      self.sink.on_chunk(
        chunk,
        Mapping {
          generated_line: mapping.generated_line,
          generated_column: mapping.generated_column,
          original: None,
        },
      );
      return;
    }

    let mut final_name_index = if name_index >= 0 {
      let name_index = name_index as u32;
      self
        .name_index_mapping
        .get(&name_index)
        .copied()
        .unwrap_or(-1)
    } else {
      -1
    };
    if final_name_index == -2 {
      let name_index = name_index as u32;
      let name = self
        .name_index_value_mapping
        .get(&name_index)
        .cloned()
        .unwrap();
      final_name_index = self.emit_name(name) as i64;
      self.name_index_mapping.insert(name_index, final_name_index);
    }
    self.sink.on_chunk(
      chunk,
      Mapping {
        generated_line: mapping.generated_line,
        generated_column: mapping.generated_column,
        original: Some(OriginalLocation {
          source_index: final_source_index as u32,
          original_line: original_line as u32,
          original_column: original_column as u32,
          name_index: (final_name_index >= 0).then_some(final_name_index as u32),
        }),
      },
    );
  }

  fn on_source(
    &mut self,
    index: u32,
    source: Cow<'source, str>,
    mut source_content: Option<&'source str>,
  ) {
    if source == self.inner_source_name {
      self.inner_source_index = index as i64;
      if let Some(inner_source) = self.inner_source {
        source_content = Some(inner_source);
      } else {
        self.inner_source = source_content;
      }
      self.source_index_mapping.insert(index, -2);
      self.record_inner_source_map(source_content.unwrap());
      return;
    }

    let global_index = self.emit_source(source, source_content);
    self.source_index_mapping.insert(index, global_index as i64);
  }

  fn on_name(&mut self, index: u32, name: Cow<'source, str>) {
    self.name_index_mapping.insert(index, -2);
    self.name_index_value_mapping.insert(index, name);
  }
}

struct InnerSourceMapSink<'sink, 'source, 'object_pool> {
  inner_source_map_line_data: &'sink mut Vec<SourceMapLineData<'source>>,
  inner_source_contents: &'sink mut LinearMap<Option<Cow<'source, str>>>,
  inner_source_content_lines:
    &'sink mut LinearMap<OnceCell<Option<SourceContentLines<'object_pool, 'source>>>>,
  inner_source_index_mapping: &'sink mut LinearMap<i64>,
  inner_source_index_value_mapping: &'sink mut InnerSourceIndexValueMapping<'source>,
  inner_name_index_mapping: &'sink mut LinearMap<i64>,
  inner_name_index_value_mapping: &'sink mut LinearMap<Cow<'source, str>>,
}

impl<'source, 'object_pool> StreamSink<'source, 'source>
  for InnerSourceMapSink<'_, 'source, 'object_pool>
{
  fn on_chunk(&mut self, chunk: Option<TextSpan<'source>>, mapping: Mapping) {
    let current_len = self.inner_source_map_line_data.len();
    let mapping_line = mapping.generated_line as usize;
    if current_len <= mapping_line {
      self
        .inner_source_map_line_data
        .reserve(mapping_line - current_len + 1);
      while self.inner_source_map_line_data.len() <= mapping_line {
        self.inner_source_map_line_data.push(SourceMapLineData {
          mappings_data: Default::default(),
          chunks: vec![],
        });
      }
    }
    let data = &mut self.inner_source_map_line_data[mapping.generated_line as usize - 1];
    data.mappings_data.reserve(5);
    data.mappings_data.push(mapping.generated_column as i64);
    data.mappings_data.push(
      mapping
        .original
        .as_ref()
        .map_or(-1, |original| original.source_index as i64),
    );
    data.mappings_data.push(
      mapping
        .original
        .as_ref()
        .map_or(-1, |original| original.original_line as i64),
    );
    data.mappings_data.push(
      mapping
        .original
        .as_ref()
        .map_or(-1, |original| original.original_column as i64),
    );
    data.mappings_data.push(
      mapping
        .original
        .and_then(|original| original.name_index)
        .map(Into::into)
        .unwrap_or(-1),
    );
    data
      .chunks
      .push(chunk.expect("inner source stream should emit chunks"));
  }

  fn on_source(
    &mut self,
    index: u32,
    source: Cow<'source, str>,
    source_content: Option<&'source str>,
  ) {
    self
      .inner_source_contents
      .insert(index, source_content.map(Cow::Borrowed));
    self
      .inner_source_content_lines
      .insert(index, Default::default());
    self.inner_source_index_mapping.insert(index, -2);
    self
      .inner_source_index_value_mapping
      .insert(index, (source, source_content));
  }

  fn on_name(&mut self, index: u32, name: Cow<'source, str>) {
    self.inner_name_index_mapping.insert(index, -2);
    self.inner_name_index_value_mapping.insert(index, name);
  }
}

pub fn stream_and_get_source_and_map<'source, 'chunk>(
  options: &MapOptions,
  object_pool: &ObjectPool,
  chunks: &'chunk dyn Chunks<'source>,
  sink: &mut dyn StreamSink<'chunk, 'source>,
) -> (GeneratedInfo, Option<SourceMapFields<'source>>) {
  let mut collecting_sink = SourceMapForwardSink {
    inner: SourceMapSink::new(options.columns),
    sink,
  };
  let generated_info = chunks.stream_with(object_pool, options, &mut collecting_sink);
  let map = collecting_sink.inner.into_map();
  (generated_info, map)
}

struct SourceMapForwardSink<'sink, 'chunk, 'source> {
  inner: SourceMapSink<'source>,
  sink: &'sink mut dyn StreamSink<'chunk, 'source>,
}

impl<'chunk, 'source> StreamSink<'chunk, 'source> for SourceMapForwardSink<'_, 'chunk, 'source> {
  #[inline]
  fn on_chunk(&mut self, chunk: Option<TextSpan<'chunk>>, mapping: Mapping) {
    self.inner.on_chunk(chunk, mapping);
    self.sink.on_chunk(chunk, mapping);
  }

  #[inline]
  fn on_source(&mut self, index: u32, source: Cow<'source, str>, content: Option<&'source str>) {
    self.inner.on_source(index, source.clone(), content);
    self.sink.on_source(index, source, content);
  }

  #[inline]
  fn on_name(&mut self, index: u32, name: Cow<'source, str>) {
    self.inner.on_name(index, name.clone());
    self.sink.on_name(index, name);
  }
}

#[cfg(test)]
mod tests {
  use std::sync::LazyLock;

  use super::{
    GeneratedInfo, StreamSink, TextSpan, split_into_potential_tokens,
    stream_chunks_of_source_map_final, stream_chunks_of_source_map_full,
    stream_chunks_of_source_map_lines_final, stream_chunks_of_source_map_lines_full,
  };
  use crate::{Mapping, ObjectPool, OriginalLocation, SourceMap};

  struct NoopSink;

  impl<'chunk, 'source> StreamSink<'chunk, 'source> for NoopSink {
    fn on_chunk(&mut self, _chunk: Option<TextSpan<'chunk>>, _mapping: Mapping) {}

    fn on_source(
      &mut self,
      _index: u32,
      _source: std::borrow::Cow<'source, str>,
      _content: Option<&'source str>,
    ) {
    }

    fn on_name(&mut self, _index: u32, _name: std::borrow::Cow<'source, str>) {}
  }

  struct ChunkSink<'chunks, 'chunk> {
    chunks: &'chunks mut Vec<(&'chunk str, Mapping)>,
  }

  impl<'chunks, 'chunk, 'source> StreamSink<'chunk, 'source> for ChunkSink<'chunks, 'chunk> {
    fn on_chunk(&mut self, chunk: Option<TextSpan<'chunk>>, mapping: Mapping) {
      self.chunks.push((chunk.unwrap().as_str(), mapping));
    }

    fn on_source(
      &mut self,
      _index: u32,
      _source: std::borrow::Cow<'source, str>,
      _content: Option<&'source str>,
    ) {
    }

    fn on_name(&mut self, _index: u32, _name: std::borrow::Cow<'source, str>) {}
  }

  const UTF16_SOURCE: &str = "var i18n = JSON.parse('{\"魑魅魍魉\":{\"en-US\":\"Evil spirits\",\"zh-CN\":\"魑魅魍魉\"}}');\nvar __webpack_exports___ = i18n[\"魑魅魍魉\"];\nexport { __webpack_exports___ as 魑魅魍魉 };";

  static UTF16_SOURCE_MAP: LazyLock<SourceMap<'static>> = LazyLock::new(|| {
    SourceMap::from_json("{\"version\":3,\"sources\":[\"i18.js\"],\"sourcesContent\":[\"var i18n = JSON.parse('{\\\"魑魅魍魉\\\":{\\\"en-US\\\":\\\"Evil spirits\\\",\\\"zh-CN\\\":\\\"魑魅魍魉\\\"}}');\\nvar __webpack_exports___ = i18n[\\\"魑魅魍魉\\\"];\\nexport { __webpack_exports___ as 魑魅魍魉 };\\n\"],\"names\":[\"i18n\",\"JSON\",\"__webpack_exports___\",\"魑魅魍魉\"],\"mappings\":\"AAAA,IAAIA,OAAOC,KAAK,KAAK,CAAC;AACtB,IAAIC,uBAAuBF,IAAI,CAAC,OAAO;AACvC,SAASE,wBAAwBC,IAAI,GAAG\"}".to_string()).unwrap()
  });

  #[test]
  fn test_stream_chunks_of_source_map_full_handles_multi_unit_utf16() {
    let source = UTF16_SOURCE;
    let source_map = UTF16_SOURCE_MAP.fields();
    let object_pool = ObjectPool::default();

    let mut chunks = vec![];
    let mut sink = ChunkSink {
      chunks: &mut chunks,
    };

    let generated_info =
      stream_chunks_of_source_map_full(&object_pool, TextSpan::new(source), source_map, &mut sink);

    assert_eq!(
      chunks,
      vec![
        (
          "var ",
          Mapping {
            generated_line: 1,
            generated_column: 0,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 1,
              original_column: 0,
              name_index: None
            })
          }
        ),
        (
          "i18n = ",
          Mapping {
            generated_line: 1,
            generated_column: 4,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 1,
              original_column: 4,
              name_index: Some(0)
            })
          }
        ),
        (
          "JSON.",
          Mapping {
            generated_line: 1,
            generated_column: 11,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 1,
              original_column: 11,
              name_index: Some(1)
            })
          }
        ),
        (
          "parse",
          Mapping {
            generated_line: 1,
            generated_column: 16,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 1,
              original_column: 16,
              name_index: None
            })
          }
        ),
        (
          "(",
          Mapping {
            generated_line: 1,
            generated_column: 21,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 1,
              original_column: 21,
              name_index: None
            })
          }
        ),
        (
          "'{\"魑魅魍魉\":{\"en-US\":\"Evil spirits\",\"zh-CN\":\"魑魅魍魉\"}}');\n",
          Mapping {
            generated_line: 1,
            generated_column: 22,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 1,
              original_column: 22,
              name_index: None
            })
          }
        ),
        (
          "var ",
          Mapping {
            generated_line: 2,
            generated_column: 0,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 2,
              original_column: 0,
              name_index: None
            })
          }
        ),
        (
          "__webpack_exports___ = ",
          Mapping {
            generated_line: 2,
            generated_column: 4,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 2,
              original_column: 4,
              name_index: Some(2)
            })
          }
        ),
        (
          "i18n",
          Mapping {
            generated_line: 2,
            generated_column: 27,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 2,
              original_column: 27,
              name_index: Some(0)
            })
          }
        ),
        (
          "[",
          Mapping {
            generated_line: 2,
            generated_column: 31,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 2,
              original_column: 31,
              name_index: None
            })
          }
        ),
        (
          "\"魑魅魍魉\"]",
          Mapping {
            generated_line: 2,
            generated_column: 32,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 2,
              original_column: 32,
              name_index: None
            })
          }
        ),
        (
          ";\n",
          Mapping {
            generated_line: 2,
            generated_column: 39,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 2,
              original_column: 39,
              name_index: None
            })
          }
        ),
        (
          "export { ",
          Mapping {
            generated_line: 3,
            generated_column: 0,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 3,
              original_column: 0,
              name_index: None
            })
          }
        ),
        (
          "__webpack_exports___ as ",
          Mapping {
            generated_line: 3,
            generated_column: 9,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 3,
              original_column: 9,
              name_index: Some(2)
            })
          }
        ),
        (
          "魑魅魍魉",
          Mapping {
            generated_line: 3,
            generated_column: 33,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 3,
              original_column: 33,
              name_index: Some(3)
            })
          }
        ),
        (
          " };",
          Mapping {
            generated_line: 3,
            generated_column: 37,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 3,
              original_column: 37,
              name_index: None
            })
          }
        )
      ]
    );

    assert_eq!(
      generated_info,
      GeneratedInfo {
        generated_line: 3,
        generated_column: 40
      }
    )
  }

  #[test]
  fn test_stream_chunks_of_source_map_final_handles_multi_unit_utf16() {
    let source = UTF16_SOURCE;
    let source_map = UTF16_SOURCE_MAP.fields();
    let mut sink = NoopSink;

    let generated_info =
      stream_chunks_of_source_map_final(TextSpan::new(source), source_map, &mut sink);

    assert_eq!(
      generated_info,
      GeneratedInfo {
        generated_line: 3,
        generated_column: 40
      }
    )
  }

  #[test]
  fn test_stream_chunks_of_source_map_lines_final_handles_multi_unit_utf16() {
    let source = UTF16_SOURCE;
    let source_map = UTF16_SOURCE_MAP.fields();
    let mut sink = NoopSink;

    let generated_info =
      stream_chunks_of_source_map_lines_final(TextSpan::new(source), source_map, &mut sink);

    assert_eq!(
      generated_info,
      GeneratedInfo {
        generated_line: 3,
        generated_column: 40
      }
    )
  }

  #[test]
  fn test_stream_chunks_of_source_map_lines_full_handles_multi_unit_utf16() {
    let source = UTF16_SOURCE;
    let source_map = UTF16_SOURCE_MAP.fields();
    let mut sink = NoopSink;

    let generated_info =
      stream_chunks_of_source_map_lines_full(TextSpan::new(source), source_map, &mut sink);

    assert_eq!(
      generated_info,
      GeneratedInfo {
        generated_line: 3,
        generated_column: 40
      }
    )
  }

  #[test]
  fn test_split_into_potential_tokens() {
    let tokens = split_into_potential_tokens("var i18n = JSON.parse('{\"魑魅魍魉\":{\"en-US\":\"Evil spirits\",\"zh-CN\":\"魑魅魍魉\"}}');\nvar __webpack_exports___ = i18n[\"魑魅魍魉\"];\nexport { __webpack_exports___ as 魑魅魍魉 };").collect::<Vec<_>>();
    assert_eq!(
      tokens,
      vec![
        "var i18n = JSON.parse('{",
        "\"魑魅魍魉\":{",
        "\"en-US\":\"Evil spirits\",\"zh-CN\":\"魑魅魍魉\"}}",
        "');\n",
        "var __webpack_exports___ = i18n[\"魑魅魍魉\"];\n",
        "export { ",
        "__webpack_exports___ as 魑魅魍魉 };",
      ]
    );
  }

  #[test]
  fn test_split_into_potential_tokens_ascii_boundaries() {
    let tokens = split_into_potential_tokens("\nfoo();\nbar { baz }\n{};\n").collect::<Vec<_>>();
    assert_eq!(tokens, vec!["\n", "foo();\n", "bar { ", "baz }\n", "{};\n"]);
  }
}
