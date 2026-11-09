use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
  io::{self, Write},
};

use crate::{
  BoxSource, CachedSource, ConcatSource, DevToolMapSource, MapOptions, OriginalSource,
  RawBufferSource, RawStringSource, ReplaceSource, Source, SourceExt, SourceMap, SourceMapSource,
  SourceValue,
  helpers::{Chunks, GeneratedInfo, StreamChunks, stream_chunks_of_raw_rope},
};

/// Generated code emitted by DevTool, with an optional
/// rendered sourceMappingURL/debugId suffix. Clones share both code and suffix.
/// No complete rendered string is retained by this wrapper.
#[derive(Clone)]
#[cfg_attr(feature = "allocative", derive(rspack_allocative::Allocative))]
pub struct DevToolCodeSource {
  inner: BoxSource,
  suffix: BoxSource,
  is_text: bool,
}

impl DevToolCodeSource {
  /// Keep ordinary UTF-8 graphs shared. Devtool previously normalized through
  /// source(), so unknown and mixed binary/text graphs retain that boundary.
  pub fn new(inner: BoxSource, suffix: BoxSource) -> Self {
    let inner = if inner.as_any().is::<RawBufferSource>() || has_utf8_text(inner.as_ref()) {
      inner
    } else {
      match inner.source() {
        SourceValue::String(text) => RawStringSource::from(text.into_owned()).boxed(),
        SourceValue::Buffer(bytes) => RawBufferSource::from(bytes.into_owned()).boxed(),
      }
    };
    let suffix = if suffix.as_any().is::<RawStringSource>() {
      suffix
    } else {
      RawStringSource::from(suffix.source().into_string_lossy().into_owned()).boxed()
    };
    // After normalization, only a raw buffer can still have binary semantics.
    let is_text = match inner.as_any().downcast_ref::<RawBufferSource>() {
      Some(buffer) => suffix.size() != 0 && std::str::from_utf8(&buffer.buffer()).is_ok(),
      None => true,
    };
    Self {
      inner,
      suffix,
      is_text,
    }
  }

  /// The shared code graph, without the appended comment.
  pub fn inner(&self) -> &BoxSource {
    &self.inner
  }

  /// The fully rendered comment, including whitespace and CSS/JS delimiters.
  pub fn suffix(&self) -> &BoxSource {
    &self.suffix
  }

  /// Whether rope, source and emitted bytes agree on UTF-8 text. A bare buffer
  /// remains a binary source, even when its bytes happen to be valid UTF-8.
  pub fn is_text(&self) -> bool {
    self.is_text
  }
}

impl Source for DevToolCodeSource {
  fn mapped(&self) -> bool {
    false
  }

  fn source(&self) -> SourceValue<'_> {
    if self.suffix.size() == 0 {
      return self.inner.source();
    }
    let mut text = String::with_capacity(self.size());
    self.rope(&mut |chunk| text.push_str(chunk));
    SourceValue::String(Cow::Owned(text))
  }

  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    self.inner.rope(on_chunk);
    if self.suffix.size() != 0 {
      self.suffix.rope(on_chunk);
    }
  }

  fn buffer(&self) -> Cow<'_, [u8]> {
    if self.suffix.size() == 0 {
      return self.inner.buffer();
    }
    let mut bytes = Vec::with_capacity(self.size());
    self
      .to_writer(&mut bytes)
      .expect("write devtool code to Vec");
    Cow::Owned(bytes)
  }

  fn size(&self) -> usize {
    self.inner.size() + self.suffix.size()
  }

  fn map(&self, _: &MapOptions) -> Option<SourceMap> {
    None
  }

  fn to_writer(&self, writer: &mut dyn Write) -> io::Result<()> {
    self.inner.to_writer(writer)?;
    self.suffix.to_writer(writer)
  }
}

impl Hash for DevToolCodeSource {
  fn hash<H: Hasher>(&self, state: &mut H) {
    "DevToolCodeSource".hash(state);
    self.inner.as_ref().hash(state);
    self.suffix.as_ref().hash(state);
  }
}

impl PartialEq for DevToolCodeSource {
  fn eq(&self, other: &Self) -> bool {
    self.inner.as_ref() == other.inner.as_ref() && self.suffix.as_ref() == other.suffix.as_ref()
  }
}
impl Eq for DevToolCodeSource {}

impl std::fmt::Debug for DevToolCodeSource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DevToolCodeSource")
      .field("inner", &self.inner)
      .field("suffix", &self.suffix)
      .finish()
  }
}

struct CodeChunks<'a>(&'a DevToolCodeSource);
impl<'source> Chunks<'source> for CodeChunks<'source> {
  fn stream<'chunk>(
    &'chunk self,
    options: &MapOptions,
    on_chunk: crate::helpers::OnChunk<'_, 'chunk>,
    _on_source: crate::helpers::OnSource<'_>,
    _on_name: crate::helpers::OnName<'_>,
  ) -> GeneratedInfo {
    stream_chunks_of_raw_rope(self.0, options, on_chunk)
  }
}
impl StreamChunks for DevToolCodeSource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a> {
    Box::new(CodeChunks(self))
  }
}

pub(crate) fn has_utf8_text(source: &dyn Source) -> bool {
  let any = source.as_any();
  if any.is::<RawStringSource>()
    || any.is::<OriginalSource>()
    || any.is::<SourceMapSource>()
    || any.is::<DevToolMapSource>()
  {
    return true;
  }
  if let Some(source) = any.downcast_ref::<RawBufferSource>() {
    return std::str::from_utf8(&source.buffer()).is_ok();
  }
  if let Some(source) = any.downcast_ref::<DevToolCodeSource>() {
    return source.is_text();
  }
  if let Some(source) = any.downcast_ref::<CachedSource>() {
    return has_utf8_text(source.inner().as_ref());
  }
  if let Some(source) = any.downcast_ref::<ConcatSource>() {
    return source.children().iter().all(|s| has_utf8_text(s.as_ref()));
  }
  if let Some(source) = any.downcast_ref::<ReplaceSource>() {
    return has_utf8_text(source.inner().as_ref());
  }
  false
}
