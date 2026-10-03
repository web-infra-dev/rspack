use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
  ops::Range,
  sync::Arc,
};

use crate::{
  BoxSource, MapOptions, ObjectPool, RawStringSource, Source, SourceExt, SourceMap, SourceValue,
  helpers::{Chunks, StreamChunks, get_map},
  replace_source::ReplaceSourceChunks,
};

/// A shared, read-only view of a source's half-open byte range.
///
/// Construction retains the source without copying its text. Text operations require
/// UTF-8 boundaries; byte operations preserve the original bytes. Source maps retain
/// the original locations of the selected text.
///
/// ```
/// use rspack_sources::{RawStringSource, Source, SourceExt, SourceSlice};
/// let original = RawStringSource::from_static("hello world").boxed();
/// let slice = SourceSlice::new(original.clone(), 6..11);
/// assert_eq!(slice.source().into_string_lossy(), "world");
/// ```
#[derive(Debug, Clone)]
pub struct SourceSlice {
  inner: BoxSource,
  range: Range<usize>,
}

impl SourceSlice {
  /// Create a view of a raw string source or any shared source.
  ///
  /// # Panics
  /// Panics if the range is reversed, outside the source, or the source exceeds
  /// the source-map byte-offset limit (`u32::MAX`). Reading borrowed text, rope chunks, or mappings panics
  /// if a nonempty slice splits a UTF-8 character.
  pub fn new(source: impl SourceExt, range: Range<usize>) -> Self {
    let inner = source.boxed();
    let size = inner.size();
    assert!(
      range.start <= range.end && range.end <= size,
      "invalid source slice range"
    );
    assert!(
      u32::try_from(size).is_ok(),
      "source slice exceeds source-map byte-offset limit"
    );
    Self { inner, range }
  }

  /// Get the source whose storage this view retains.
  pub fn inner(&self) -> &BoxSource {
    &self.inner
  }

  /// Get the selected half-open byte range.
  pub fn range(&self) -> Range<usize> {
    self.range.clone()
  }
}

impl Source for SourceSlice {
  fn source(&self) -> SourceValue<'_> {
    if let Some(raw) = self
      .inner
      .as_ref()
      .as_any()
      .downcast_ref::<RawStringSource>()
    {
      return SourceValue::String(Cow::Borrowed(&raw.as_str()[self.range.clone()]));
    }
    SourceValue::String(SourceValue::Buffer(self.buffer()).into_string_lossy())
  }

  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    let mut position = 0;
    self.inner.rope(&mut |chunk| {
      let end = position + chunk.len();
      let start = self.range.start.max(position);
      let selected_end = self.range.end.min(end);
      if start < selected_end {
        on_chunk(&chunk[start - position..selected_end - position]);
      }
      position = end;
    });
  }

  fn buffer(&self) -> Cow<'_, [u8]> {
    if let Some(raw) = self
      .inner
      .as_ref()
      .as_any()
      .downcast_ref::<RawStringSource>()
    {
      return Cow::Borrowed(&raw.as_str().as_bytes()[self.range.clone()]);
    }
    let mut bytes = Vec::with_capacity(self.size());
    self
      .to_writer(&mut bytes)
      .expect("writing a source slice to a Vec cannot fail");
    Cow::Owned(bytes)
  }

  fn size(&self) -> usize {
    self.range.len()
  }

  fn map(&self, object_pool: &ObjectPool, options: &MapOptions) -> Option<SourceMap<'_>> {
    get_map(object_pool, self.stream_chunks().as_ref(), options).map(SourceMap::from_fields)
  }

  fn map_static(
    self: Arc<Self>,
    object_pool: &ObjectPool,
    options: &MapOptions,
  ) -> Option<SourceMap<'static>> {
    self
      .map(object_pool, options)
      .map(|map| map.into_static(self.clone()))
  }

  fn to_writer(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    self.inner.to_writer(&mut SliceWriter {
      writer,
      range: self.range.clone(),
      position: 0,
    })
  }
}

impl StreamChunks for SourceSlice {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a> {
    Box::new(ReplaceSourceChunks::slice(&self.inner, self.range.clone()))
  }
}

impl PartialEq for SourceSlice {
  fn eq(&self, other: &Self) -> bool {
    self.inner.as_ref() == other.inner.as_ref() && self.range == other.range
  }
}

impl Eq for SourceSlice {}

impl Hash for SourceSlice {
  fn hash<H: Hasher>(&self, state: &mut H) {
    "SourceSlice".hash(state);
    self.inner.hash(state);
    self.range.hash(state);
  }
}

struct SliceWriter<'a> {
  writer: &'a mut dyn std::io::Write,
  range: Range<usize>,
  position: usize,
}

impl std::io::Write for SliceWriter<'_> {
  fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
    let end = self.position + bytes.len();
    let start = self.range.start.max(self.position);
    let selected_end = self.range.end.min(end);
    if start < selected_end {
      self
        .writer
        .write_all(&bytes[start - self.position..selected_end - self.position])?;
    }
    self.position = end;
    Ok(bytes.len())
  }

  fn flush(&mut self) -> std::io::Result<()> {
    self.writer.flush()
  }
}
