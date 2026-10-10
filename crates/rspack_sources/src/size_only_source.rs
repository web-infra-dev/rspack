use std::{borrow::Cow, hash::Hasher, sync::Arc};

use crate::{
  MapOptions, ObjectPool, Source, SourceMap, SourceValue,
  helpers::{Chunks, StreamChunks},
};

const CONTENT_UNAVAILABLE: &str =
  "Content and Map of this Source is not available (only size() is supported)";

/// An emitted asset whose content is no longer retained by the compilation.
///
/// Only `size()` is supported. Content, mapping and content-hash operations panic;
/// bindings must expose this as a size-only source rather than request its content.
#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "allocative", derive(allocative::Allocative))]
pub struct SizeOnlySource {
  size: usize,
}

impl SizeOnlySource {
  /// Create a placeholder retaining only the emitted byte length.
  pub fn new(size: usize) -> Self {
    Self { size }
  }
}

impl Source for SizeOnlySource {
  fn size(&self) -> usize {
    self.size
  }

  fn source(&self) -> SourceValue<'_> {
    panic!("{CONTENT_UNAVAILABLE}")
  }

  fn rope<'a>(&'a self, _: &mut dyn FnMut(&'a str)) {
    panic!("{CONTENT_UNAVAILABLE}")
  }

  fn buffer(&self) -> Cow<'_, [u8]> {
    panic!("{CONTENT_UNAVAILABLE}")
  }

  fn map(&self, _: &ObjectPool, _: &MapOptions) -> Option<SourceMap<'_>> {
    panic!("{CONTENT_UNAVAILABLE}")
  }

  fn map_static(self: Arc<Self>, _: &ObjectPool, _: &MapOptions) -> Option<SourceMap<'static>> {
    panic!("{CONTENT_UNAVAILABLE}")
  }

  fn update_hash(&self, _: &mut dyn Hasher) {
    panic!("{CONTENT_UNAVAILABLE}")
  }

  fn to_writer(&self, _: &mut dyn std::io::Write) -> std::io::Result<()> {
    Err(std::io::Error::other(CONTENT_UNAVAILABLE))
  }
}

impl StreamChunks for SizeOnlySource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a> {
    panic!("{CONTENT_UNAVAILABLE}")
  }
}
