use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
  io::{self, Write},
  sync::{Arc, OnceLock},
};

use aho_corasick::{AhoCorasick, Match};
use rspack_core::rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, MappingSource, OriginalSource,
  RawBufferSource, RawStringSource, ReplaceSource, Source, SourceExt, SourceMap, SourceMapSource,
  SourceValue,
  stream_chunks::{
    Chunks, GeneratedInfo, OnChunk, OnName, OnSource, StreamChunks, stream_chunks_of_raw_rope,
  },
};
use rspack_util::streaming_matcher::StreamingMatcher;

/// A content-only view used by real content hashing and final asset emission.
///
/// Clones share the input and edits. The view is created after render/devtool cache
/// entries have captured their inputs; it is not a persistent cache payload.
#[derive(Clone)]
pub struct RealContentHashSource(Arc<RealContentHashSourceData>);

struct RealContentHashSourceData {
  inner: RealContentHashInput,
  replacements: Vec<(usize, usize, String)>,
  // Previously, any rewritten ordinary source became a RawStringSource, even
  // when all replacements happened to preserve their original hash strings.
  normalized: bool,
  size: OnceLock<usize>,
}

#[derive(Clone)]
enum RealContentHashInput {
  StreamingText(BoxSource),
  Binary(BoxSource),
  // source() text can differ from writer bytes, e.g. for non-UTF-8 buffer
  // children in ConcatSource. Scan/edit the text, but keep untouched output
  // and source-map identity on the original source.
  MaterializedText {
    original: BoxSource,
    text: BoxSource,
  },
}

impl RealContentHashInput {
  fn new(source: BoxSource) -> Self {
    match content_kind(source.as_ref()) {
      Some(false) => Self::StreamingText(source),
      Some(true) => Self::Binary(source),
      None => {
        let text = match source.source() {
          SourceValue::String(text) => Some(RawStringSource::from(text.into_owned()).boxed()),
          SourceValue::Buffer(_) => None,
        };
        match text {
          Some(text) => Self::MaterializedText {
            original: source,
            text,
          },
          None => Self::Binary(source),
        }
      }
    }
  }

  fn original(&self) -> &BoxSource {
    match self {
      Self::StreamingText(source) | Self::Binary(source) => source,
      Self::MaterializedText { original, .. } => original,
    }
  }

  fn content(&self) -> &BoxSource {
    match self {
      Self::StreamingText(source) | Self::Binary(source) => source,
      Self::MaterializedText { text, .. } => text,
    }
  }
}

impl RealContentHashSource {
  pub fn new(inner: BoxSource) -> Self {
    Self(Arc::new(RealContentHashSourceData {
      inner: RealContentHashInput::new(inner),
      replacements: Vec::new(),
      normalized: false,
      size: OnceLock::new(),
    }))
  }

  pub fn is_text(&self) -> bool {
    !matches!(self.0.inner, RealContentHashInput::Binary(_))
  }

  /// Whether byte streaming agrees with the source's UTF-8 text view.
  pub fn can_stream_as_text(&self) -> bool {
    self.is_text()
      && (matches!(self.0.inner, RealContentHashInput::StreamingText(_))
        || self.0.normalized
        || !self.0.replacements.is_empty())
  }

  /// Find non-overlapping matches, including patterns split across writer calls.
  pub fn matches(&self, matcher: &AhoCorasick) -> (Vec<Match>, usize) {
    if !self.is_text() {
      return (Vec::new(), self.size());
    }
    let (matches, size) = if let Some(bytes) = borrowed_bytes(self.input().as_ref()) {
      (matcher.find_iter(bytes).collect(), bytes.len())
    } else {
      let mut matches = Vec::new();
      let mut writer = StreamingMatcher::new(matcher, |found| matches.push(found));
      self
        .input()
        .to_writer(&mut writer)
        .expect("scan real content hash source");
      let size = writer.finish();
      (matches, size)
    };
    if matches!(self.0.inner, RealContentHashInput::StreamingText(_)) {
      let _ = self.0.size.set(size);
    }
    (matches, size)
  }

  /// Create a view of sorted, non-overlapping byte edits in the scanned input.
  pub fn with_replacements(
    &self,
    replacements: Vec<(usize, usize, String)>,
    input_size: usize,
    normalized: bool,
  ) -> Self {
    let mut end = 0;
    let mut size = input_size;
    for (start, stop, text) in &replacements {
      assert!(end <= *start && start <= stop && *stop <= input_size);
      size = size - (stop - start) + text.len();
      end = *stop;
    }
    if !normalized && replacements.is_empty() {
      return self.clone();
    }
    let inner = if self.0.replacements.is_empty() && !self.0.normalized {
      self.0.inner.clone()
    } else if self.is_text() {
      RealContentHashInput::StreamingText(self.clone().boxed())
    } else {
      RealContentHashInput::Binary(self.clone().boxed())
    };
    Self(Arc::new(RealContentHashSourceData {
      inner,
      replacements,
      normalized,
      size: OnceLock::from(size),
    }))
  }

  /// Unmodified assets keep their original source-map and cache identity.
  pub fn into_source(self) -> BoxSource {
    if self.0.normalized || !self.0.replacements.is_empty() {
      self.boxed()
    } else {
      Arc::clone(self.0.inner.original())
    }
  }

  pub fn original_is_mapping(&self) -> bool {
    self.0.inner.original().as_any().is::<MappingSource>()
      || self
        .0
        .inner
        .original()
        .as_any()
        .downcast_ref::<Self>()
        .is_some_and(Self::original_is_mapping)
  }

  /// Preserve RCH's former RawStringSource normalization when deduplicating
  /// adjacent assets. Untouched non-raw sources retain structural equality.
  pub fn eq_for_hash(&self, other: &Self) -> bool {
    let by_content = |s: &Self| {
      s.0.normalized
        || !s.0.replacements.is_empty()
        || s.original_is_mapping()
        || s.0.inner.original().as_any().is::<RawStringSource>()
    };
    match (by_content(self), by_content(other)) {
      (true, true) => self.content_eq(other),
      (false, false) => self.0.inner.original().as_ref() == other.0.inner.original().as_ref(),
      _ => false,
    }
  }

  pub fn content_eq(&self, other: &Self) -> bool {
    if Arc::ptr_eq(&self.0, &other.0) {
      return true;
    }
    if self.size() != other.size() {
      return false;
    }
    let mut writer = CompareWriter {
      chunks: self.byte_chunks(),
      chunk: &[],
      offset: 0,
    };
    other.to_writer(&mut writer).is_ok() && writer.is_finished()
  }

  pub fn update_content_hash(&self, hasher: &mut dyn Hasher) {
    if self.0.replacements.is_empty()
      && let Some(bytes) = borrowed_bytes(self.output_input().as_ref())
    {
      hasher.write(bytes);
      return;
    }
    let mut writer = io::BufWriter::with_capacity(64 * 1024, HashWriter(hasher));
    self
      .to_writer(&mut writer)
      .expect("hash real content hash source");
    writer.flush().expect("flush real content hash source hash");
  }

  fn input(&self) -> &BoxSource {
    self.0.inner.content()
  }

  fn output_input(&self) -> &BoxSource {
    if self.0.normalized || !self.0.replacements.is_empty() {
      self.input()
    } else {
      self.0.inner.original()
    }
  }

  fn byte_chunks(&self) -> Box<dyn Iterator<Item = &[u8]> + '_> {
    let chunks = source_chunks(self.output_input().as_ref());
    if self.0.replacements.is_empty() {
      chunks
    } else {
      Box::new(EditedChunks {
        inner: chunks,
        replacements: &self.0.replacements,
        chunk: &[],
        position: 0,
        skip_until: 0,
      })
    }
  }
}

// Some(false): known UTF-8 text; Some(true): source() returns a binary buffer.
// None preserves the previous materializing fallback for unknown implementations
// and composite text whose byte writer differs from its lossy rope view.
fn content_kind(source: &dyn Source) -> Option<bool> {
  let any = source.as_any();
  if any.is::<RawStringSource>()
    || any.is::<OriginalSource>()
    || any.is::<SourceMapSource>()
    || any.is::<MappingSource>()
  {
    return Some(false);
  }
  if any.is::<RawBufferSource>() {
    return Some(true);
  }
  if let Some(s) = any.downcast_ref::<RealContentHashSource>() {
    return if !s.is_text() {
      Some(true)
    } else {
      s.can_stream_as_text().then_some(false)
    };
  }
  if let Some(s) = any.downcast_ref::<ConcatSource>() {
    let children = s.children();
    if children.len() == 1 {
      return content_kind(children[0].as_ref());
    }
    return children
      .iter()
      .all(|s| utf8_writer(s.as_ref()))
      .then_some(false);
  }
  if let Some(s) = any.downcast_ref::<CachedSource>() {
    if s.inner().as_any().is::<RawBufferSource>() {
      return Some(true);
    }
    return utf8_writer(s.inner().as_ref()).then_some(false);
  }
  if let Some(s) = any.downcast_ref::<ReplaceSource>() {
    if s.replacements().is_empty() {
      return content_kind(s.inner().as_ref());
    }
    return Some(false);
  }
  None
}

fn utf8_writer(source: &dyn Source) -> bool {
  if let Some(buffer) = source.as_any().downcast_ref::<RawBufferSource>() {
    return std::str::from_utf8(&buffer.buffer()).is_ok();
  }
  if let Some(cached) = source.as_any().downcast_ref::<CachedSource>() {
    return utf8_writer(cached.inner().as_ref());
  }
  content_kind(source) == Some(false)
}

fn borrowed_bytes(source: &dyn Source) -> Option<&[u8]> {
  let any = source.as_any();
  if let Some(source) = any.downcast_ref::<OriginalSource>() {
    return Some(source.value().as_bytes());
  }
  if let Some(source) = any.downcast_ref::<SourceMapSource>() {
    return Some(source.value().as_bytes());
  }
  if any.is::<RawStringSource>() || any.is::<RawBufferSource>() {
    let Cow::Borrowed(bytes) = source.buffer() else {
      unreachable!("raw sources borrow their bytes")
    };
    return Some(bytes);
  }
  None
}

fn source_chunks(source: &dyn Source) -> Box<dyn Iterator<Item = &[u8]> + '_> {
  if let Some(source) = source.as_any().downcast_ref::<RealContentHashSource>() {
    return source.byte_chunks();
  }
  if let Some(mapping) = source.as_any().downcast_ref::<MappingSource>() {
    return Box::new(mapping.json_chunks_iter().map(str::as_bytes));
  }
  if let Some(bytes) = borrowed_bytes(source) {
    return Box::new(std::iter::once(bytes));
  }
  if let Some(source) = source.as_any().downcast_ref::<CachedSource>() {
    return source_chunks(source.inner().as_ref());
  }
  if let Some(concat) = source.as_any().downcast_ref::<ConcatSource>() {
    return Box::new(
      concat
        .children()
        .iter()
        .flat_map(|s| source_chunks(s.as_ref())),
    );
  }
  // rope() lends source-owned slices. Retain references rather than rendered
  // bytes for ReplaceSource and existing/custom leaf implementations.
  let mut chunks = Vec::new();
  source.rope(&mut |chunk| chunks.push(chunk.as_bytes()));
  Box::new(chunks.into_iter())
}

struct EditedChunks<'a> {
  inner: Box<dyn Iterator<Item = &'a [u8]> + 'a>,
  replacements: &'a [(usize, usize, String)],
  chunk: &'a [u8],
  position: usize,
  skip_until: usize,
}

impl<'a> Iterator for EditedChunks<'a> {
  type Item = &'a [u8];
  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if self.position >= self.skip_until
        && let Some((start, stop, text)) = self.replacements.first()
        && *start == self.position
      {
        self.skip_until = *stop;
        self.replacements = &self.replacements[1..];
        return Some(text.as_bytes());
      }
      if self.chunk.is_empty() {
        match self.inner.next() {
          Some(chunk) => self.chunk = chunk,
          None => {
            let (_, stop, text) = self.replacements.first()?;
            self.skip_until = *stop;
            self.replacements = &self.replacements[1..];
            return Some(text.as_bytes());
          }
        }
      }
      if self.position < self.skip_until {
        let n = self.chunk.len().min(self.skip_until - self.position);
        self.chunk = &self.chunk[n..];
        self.position += n;
        continue;
      }
      let n = self.replacements.first().map_or(self.chunk.len(), |r| {
        self.chunk.len().min(r.0 - self.position)
      });
      let chunk = &self.chunk[..n];
      self.chunk = &self.chunk[n..];
      self.position += n;
      return Some(chunk);
    }
  }
}

struct ReplacementWriter<'a> {
  writer: &'a mut dyn Write,
  replacements: &'a [(usize, usize, String)],
  position: usize,
  consumed: usize,
}
impl ReplacementWriter<'_> {
  fn finish(self) -> io::Result<()> {
    for (_, _, text) in self.replacements {
      self.writer.write_all(text.as_bytes())?;
    }
    Ok(())
  }
}
impl Write for ReplacementWriter<'_> {
  fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
    let end = self.position + bytes.len();
    while let Some((start, stop, text)) = self.replacements.first()
      && *start <= end
    {
      if *start > self.consumed.max(self.position) {
        self.writer.write_all(
          &bytes[self.consumed.max(self.position) - self.position..start - self.position],
        )?;
      }
      self.writer.write_all(text.as_bytes())?;
      self.consumed = *stop;
      self.replacements = &self.replacements[1..];
    }
    if self.consumed < end {
      self
        .writer
        .write_all(&bytes[self.consumed.max(self.position) - self.position..])?;
      self.consumed = end;
    }
    self.position = end;
    Ok(bytes.len())
  }
  fn flush(&mut self) -> io::Result<()> {
    self.writer.flush()
  }
}

struct CompareWriter<'a> {
  chunks: Box<dyn Iterator<Item = &'a [u8]> + 'a>,
  chunk: &'a [u8],
  offset: usize,
}
impl CompareWriter<'_> {
  fn is_finished(&mut self) -> bool {
    self.offset == self.chunk.len() && self.chunks.all(|c| c.is_empty())
  }
}
impl Write for CompareWriter<'_> {
  fn write(&mut self, mut bytes: &[u8]) -> io::Result<usize> {
    let len = bytes.len();
    while !bytes.is_empty() {
      while self.offset == self.chunk.len() {
        self.chunk = self
          .chunks
          .next()
          .ok_or_else(|| io::Error::other("source contents differ"))?;
        self.offset = 0;
      }
      let n = bytes.len().min(self.chunk.len() - self.offset);
      if bytes[..n] != self.chunk[self.offset..self.offset + n] {
        return Err(io::Error::other("source contents differ"));
      }
      self.offset += n;
      bytes = &bytes[n..];
    }
    Ok(len)
  }
  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}
struct HashWriter<'a>(&'a mut dyn Hasher);
impl Write for HashWriter<'_> {
  fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
    self.0.write(bytes);
    Ok(bytes.len())
  }
  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

impl Source for RealContentHashSource {
  fn source(&self) -> SourceValue<'_> {
    if self.0.replacements.is_empty() {
      return self.output_input().source();
    }
    SourceValue::String(Cow::Owned(
      String::from_utf8(self.buffer().into_owned()).expect("RCH text must be UTF-8"),
    ))
  }
  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    if self.0.replacements.is_empty() {
      self.output_input().rope(on_chunk);
      return;
    }
    for chunk in self.byte_chunks() {
      on_chunk(std::str::from_utf8(chunk).expect("RCH edits preserve UTF-8 boundaries"));
    }
  }
  fn buffer(&self) -> Cow<'_, [u8]> {
    if self.0.replacements.is_empty() {
      return self.output_input().buffer();
    }
    let mut bytes = Vec::with_capacity(self.size());
    self
      .to_writer(&mut bytes)
      .expect("write real content hash source");
    Cow::Owned(bytes)
  }
  fn size(&self) -> usize {
    *self.0.size.get_or_init(|| self.output_input().size())
  }
  fn map(&self, _: &MapOptions) -> Option<SourceMap<'_>> {
    None
  }
  fn map_static(self: Arc<Self>, _: &MapOptions) -> Option<SourceMap<'static>> {
    None
  }
  fn to_writer(&self, writer: &mut dyn Write) -> io::Result<()> {
    if self.0.replacements.is_empty() {
      return self.output_input().to_writer(writer);
    }
    let mut writer = ReplacementWriter {
      writer,
      replacements: &self.0.replacements,
      position: 0,
      consumed: 0,
    };
    self.output_input().to_writer(&mut writer)?;
    writer.finish()
  }
}
struct RealContentHashChunks<'a>(&'a RealContentHashSource);
impl<'source> Chunks<'source> for RealContentHashChunks<'source> {
  fn stream<'chunk>(
    &'chunk self,
    options: &MapOptions,
    on_chunk: OnChunk<'_, 'chunk>,
    _on_source: OnSource<'_, 'source>,
    _on_name: OnName<'_, 'source>,
  ) -> GeneratedInfo {
    stream_chunks_of_raw_rope(self.0, options, on_chunk)
  }
}
impl StreamChunks for RealContentHashSource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a> {
    Box::new(RealContentHashChunks(self))
  }
}
impl Hash for RealContentHashSource {
  fn hash<H: Hasher>(&self, state: &mut H) {
    "RealContentHashSource".hash(state);
    self.0.inner.original().as_ref().hash(state);
    self.0.replacements.hash(state);
    self.0.normalized.hash(state);
  }
}
impl PartialEq for RealContentHashSource {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.0, &other.0)
      || (self.0.inner.original().as_ref() == other.0.inner.original().as_ref()
        && self.0.replacements == other.0.replacements
        && self.0.normalized == other.0.normalized)
  }
}
impl Eq for RealContentHashSource {}
impl std::fmt::Debug for RealContentHashSource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RealContentHashSource")
      .field("replacements", &self.0.replacements)
      .finish_non_exhaustive()
  }
}
