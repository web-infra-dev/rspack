#![allow(missing_docs)]
use std::{borrow::Cow, hash::Hash};

use rspack_sources::{
  ConcatSource, MapOptions, ObjectPool, RawStringSource, Source, SourceExt, SourceMap, SourceValue,
  stream_chunks::{
    Chunks, GeneratedInfo, OnChunk, OnName, OnSource, StreamChunks, stream_chunks_default,
  },
};

#[derive(Debug, Eq)]
struct CompatSource(&'static str, Option<SourceMap>);

impl Source for CompatSource {
  fn source(&self) -> SourceValue<'_> {
    SourceValue::String(Cow::Borrowed(self.0))
  }

  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    on_chunk(self.0)
  }

  fn buffer(&self) -> Cow<'_, [u8]> {
    Cow::Borrowed(self.0.as_bytes())
  }

  fn size(&self) -> usize {
    42
  }

  fn map(&self, _object_pool: &ObjectPool, _options: &MapOptions) -> Option<SourceMap> {
    self.1.clone()
  }

  fn to_writer(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    writer.write_all(self.0.as_bytes())
  }
}

struct CompatSourceChunks<'source>(&'static str, Option<&'source SourceMap>);

impl<'source> CompatSourceChunks<'source> {
  pub fn new(source: &'source CompatSource) -> Self {
    CompatSourceChunks(source.0, source.1.as_ref())
  }
}

impl Chunks for CompatSourceChunks<'_> {
  fn stream<'a>(
    &'a self,
    object_pool: &'a ObjectPool,
    options: &MapOptions,
    on_chunk: OnChunk<'_, 'a>,
    on_source: OnSource<'_, 'a>,
    on_name: OnName<'_, 'a>,
  ) -> GeneratedInfo {
    stream_chunks_default(
      options,
      object_pool,
      self.0,
      self.1,
      on_chunk,
      on_source,
      on_name,
    )
  }
}

impl StreamChunks for CompatSource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks + 'a> {
    Box::new(CompatSourceChunks::new(self))
  }
}

impl Hash for CompatSource {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    "__CompatSource".hash(state);
    self.0.hash(state);
  }
}

impl PartialEq for CompatSource {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl Clone for CompatSource {
  fn clone(&self) -> Self {
    Self(self.0, self.1.clone())
  }
}

#[test]
fn should_work_with_custom_compat_source() {
  const CONTENT: &str = "Line1\n\nLine3\n";

  let source = CompatSource(CONTENT, None);
  assert_eq!(source.source().into_string_lossy(), CONTENT);
  assert_eq!(source.size(), 42);
  assert_eq!(source.buffer(), CONTENT.as_bytes());
  assert_eq!(
    source.map(&ObjectPool::default(), &MapOptions::default()),
    None
  );
}

#[test]
fn should_generate_correct_source_map() {
  let source_map = SourceMap::from_json(
    r#"{
      "version": 3,
      "sources": ["compat.js"],
      "sourcesContent": ["Line1\n\nLine3\n"],
      "mappings": "AAAA;AACA;AACA",
      "names": []
    }"#,
  )
  .unwrap();

  let result = ConcatSource::new([
    RawStringSource::from("Line0\n").boxed(),
    CompatSource("Line1\nLine2\nLine3\n", Some(source_map)).boxed(),
  ]);

  let source = result.source();
  let map = result
    .map(&ObjectPool::default(), &MapOptions::default())
    .unwrap();

  let expected_source = "Line0\nLine1\nLine2\nLine3\n";
  let expected_source_map = SourceMap::from_json(
    r#"{
      "version": 3,
      "sources": ["compat.js"],
      "sourcesContent": ["Line1\n\nLine3\n"],
      "mappings": ";AAAA;AACA;AACA",
      "names": []
    }"#,
  )
  .unwrap();

  assert_eq!(source.into_string_lossy(), expected_source);
  assert_eq!(map, expected_source_map)
}
