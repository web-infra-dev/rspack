use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
  sync::Arc,
};

use crate::{
  BoxSource, CachedSource, ConcatSource, MapOptions, MappingSource, OriginalSource,
  RawBufferSource, RawStringSource, ReplaceSource, Source, SourceExt, SourceMap, SourceMapSource,
  SourceValue,
  helpers::{Chunks, GeneratedInfo, StreamChunks, stream_chunks_of_raw_rope},
};

/// A source that preserves content but never exposes a source map.
///
/// Devtool emits an asset's map as a separate asset. Keeping the rendered
/// source graph instead of flattening it avoids a full copy of the asset text,
/// while this wrapper keeps `source.map()` behavior identical to the flattened
/// source that had no map. Binary/text composites and unknown implementations
/// are normalized through `source()` when byte equivalence cannot be established.
#[derive(Clone)]
pub struct SourceOnlySource(BoxSource);

impl SourceOnlySource {
  pub fn new(inner: BoxSource) -> Self {
    // Devtool used to store source(), not buffer() or writer output. Those can
    // differ for binary/text composites and custom sources. Preserve the graph
    // only when its byte APIs agree with that value.
    let inner = if inner.as_any().is::<RawBufferSource>() || has_utf8_text(inner.as_ref()) {
      inner
    } else {
      match inner.source() {
        SourceValue::String(text) => RawStringSource::from(text.into_owned()).boxed(),
        SourceValue::Buffer(bytes) => RawBufferSource::from(bytes.into_owned()).boxed(),
      }
    };
    Self(inner)
  }

  pub fn inner(&self) -> &BoxSource {
    &self.0
  }
}

impl Source for SourceOnlySource {
  fn source(&self) -> SourceValue<'_> {
    self.0.source()
  }

  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    self.0.rope(on_chunk)
  }

  fn buffer(&self) -> Cow<'_, [u8]> {
    self.0.buffer()
  }

  fn size(&self) -> usize {
    self.0.size()
  }

  fn map(&self, _: &MapOptions) -> Option<SourceMap<'_>> {
    None
  }

  fn map_static(self: Arc<Self>, _: &MapOptions) -> Option<SourceMap<'static>> {
    None
  }

  fn to_writer(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    self.0.to_writer(writer)
  }
}

impl StreamChunks for SourceOnlySource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a> {
    Box::new(SourceOnlySourceChunks(self))
  }
}

struct SourceOnlySourceChunks<'a>(&'a SourceOnlySource);

impl<'source> Chunks<'source> for SourceOnlySourceChunks<'source> {
  fn stream<'chunk>(
    &'chunk self,
    options: &MapOptions,
    on_chunk: crate::helpers::OnChunk<'_, 'chunk>,
    _on_source: crate::helpers::OnSource<'_, 'source>,
    _on_name: crate::helpers::OnName<'_, 'source>,
  ) -> GeneratedInfo {
    stream_chunks_of_raw_rope(self.0.inner().as_ref(), options, on_chunk)
  }
}

impl Hash for SourceOnlySource {
  fn hash<H: Hasher>(&self, state: &mut H) {
    "SourceOnlySource".hash(state);
    self.0.as_ref().hash(state);
  }
}

impl PartialEq for SourceOnlySource {
  fn eq(&self, other: &Self) -> bool {
    self.0.as_ref() == other.0.as_ref()
  }
}

impl Eq for SourceOnlySource {}

impl std::fmt::Debug for SourceOnlySource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("SourceOnlySource").field(&self.0).finish()
  }
}

// A conservative check: unknown implementations retain the old source()
// materialization boundary rather than assuming writer/text equivalence.
fn has_utf8_text(source: &dyn Source) -> bool {
  let any = source.as_any();
  if any.is::<RawStringSource>()
    || any.is::<OriginalSource>()
    || any.is::<SourceMapSource>()
    || any.is::<MappingSource>()
  {
    return true;
  }
  if let Some(source) = any.downcast_ref::<RawBufferSource>() {
    return std::str::from_utf8(&source.buffer()).is_ok();
  }
  if let Some(source) = any.downcast_ref::<SourceOnlySource>() {
    return has_utf8_text(source.inner().as_ref());
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
