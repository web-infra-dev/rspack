use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
  io::{self, Write},
};

use crate::{
  BoxSource, MapOptions, RawStringSource, Source, SourceExt, SourceMap, SourceValue,
  helpers::{Chunks, RawRopeChunks, StreamChunks},
};

/// Generated code emitted by DevTool, with an optional
/// rendered sourceMappingURL/debugId suffix. Clones share both code and suffix.
/// No complete rendered string is retained by this wrapper.
#[derive(Clone)]
#[cfg_attr(feature = "allocative", derive(rspack_allocative::Allocative))]
pub struct DevToolCodeSource {
  inner: BoxSource,
  suffix: BoxSource,
}

impl DevToolCodeSource {
  /// Wrap generated text after DevTool has collected its source map.
  pub fn new(inner: BoxSource, suffix: BoxSource) -> Self {
    let suffix = if suffix.as_any().is::<RawStringSource>() {
      suffix
    } else {
      RawStringSource::from(suffix.source().into_string_lossy().into_owned()).boxed()
    };
    Self { inner, suffix }
  }

  /// The shared code graph, without the appended comment.
  pub fn inner(&self) -> &BoxSource {
    &self.inner
  }

  /// The fully rendered comment, including whitespace and CSS/JS delimiters.
  pub fn suffix(&self) -> &BoxSource {
    &self.suffix
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

impl StreamChunks for DevToolCodeSource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a> {
    Box::new(RawRopeChunks(self))
  }
}
