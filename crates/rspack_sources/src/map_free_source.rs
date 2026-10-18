use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
  sync::Arc,
};

use crate::{
  BoxSource, MapOptions, Source, SourceMap, SourceValue,
  helpers::{Chunks, StreamChunks},
};

/// A source that preserves content but never exposes a source map.
///
/// Devtool emits an asset's map as a separate asset. Keeping the rendered
/// source graph instead of flattening it avoids a full copy of the asset text,
/// while this wrapper keeps `source.map()` behavior identical to the flattened
/// source that had no map.
#[derive(Clone)]
pub struct MapFreeSource(BoxSource);

impl MapFreeSource {
  pub fn new(inner: BoxSource) -> Self {
    Self(inner)
  }

  pub fn inner(&self) -> &BoxSource {
    &self.0
  }
}

impl Source for MapFreeSource {
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

  fn release_rope_caches(&self) {
    self.0.release_rope_caches();
  }
}

impl StreamChunks for MapFreeSource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a> {
    self.0.stream_chunks()
  }
}

impl Hash for MapFreeSource {
  fn hash<H: Hasher>(&self, state: &mut H) {
    "MapFreeSource".hash(state);
    self.0.as_ref().hash(state);
  }
}

impl PartialEq for MapFreeSource {
  fn eq(&self, other: &Self) -> bool {
    self.0.as_ref() == other.0.as_ref()
  }
}

impl Eq for MapFreeSource {}

impl std::fmt::Debug for MapFreeSource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("MapFreeSource").field(&self.0).finish()
  }
}
