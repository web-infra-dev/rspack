use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
  io::{self, Write},
  sync::Arc,
};

use rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, ObjectPool, OriginalSource, RawBufferSource,
  RawStringSource, ReplaceSource, ReplacementEnforce, Source, SourceExt, SourceMap, SourceSlice,
  SourceValue,
  stream_chunks::{Chunks, StreamChunks},
};
use smol_str::SmolStr;

fn hash(source: &dyn Source) -> u64 {
  let mut hasher = std::collections::hash_map::DefaultHasher::new();
  source.update_hash(&mut hasher);
  hasher.finish()
}

fn assert_text(source: &dyn Source, expected: &str) {
  assert_eq!(source.source().into_string_lossy(), expected);
  assert_eq!(source.buffer().as_ref(), expected.as_bytes());
  assert_eq!(source.size(), expected.len());
  let mut rope = String::new();
  source.rope(&mut |chunk| rope.push_str(chunk));
  assert_eq!(rope, expected);
  let mut bytes = Vec::new();
  source.to_writer(&mut bytes).expect("write source bytes");
  assert_eq!(bytes, expected.as_bytes());
}

#[test]
fn raw_string_storage_is_borrowed_and_hashes_by_text() {
  let shared: Arc<str> = Arc::from("shared text beyond small string inline capacity");
  let raw = RawStringSource::from(shared.clone());
  assert_eq!(raw.as_str().as_ptr(), shared.as_ptr());
  let cloned = raw.clone().boxed();
  assert_eq!(cloned.source().as_bytes().as_ptr(), shared.as_ptr());
  let owned = String::from("owned text");
  let ptr = owned.as_ptr();
  let raw_owned = RawStringSource::from(owned);
  assert_eq!(raw_owned.as_str().as_ptr(), ptr);
  let small = SmolStr::new(shared.as_ref());
  let ptr = small.as_ptr();
  let raw_small = RawStringSource::from(small);
  assert_eq!(raw_small.as_str().as_ptr(), ptr);
  for raw in [
    raw,
    raw_small,
    RawStringSource::from(shared.to_string()),
    RawStringSource::from_static("shared text beyond small string inline capacity"),
  ] {
    assert_text(&raw, &shared);
    assert_eq!(hash(&raw), hash(&RawStringSource::from(shared.as_ref())));
    assert_eq!(raw, RawStringSource::from(shared.as_ref()));
  }
  assert_text(&RawStringSource::from(SmolStr::new("small")), "small");
}

#[test]
fn slices_share_storage_and_preserve_unicode_mappings() {
  let raw = RawStringSource::from("prefix😀你好\nsuffix".to_string()).boxed();
  let slice = SourceSlice::new(raw.clone(), 6..17);
  assert_text(&slice, "😀你好\n");
  assert_eq!(
    slice.source().as_bytes().as_ptr(),
    raw.source().as_bytes()[6..].as_ptr()
  );
  let original = OriginalSource::new("😀你好 world\nnext", "input.js").boxed();
  let slice = SourceSlice::new(original, 4..16);
  assert_text(&slice, "你好 world");
  let pool = ObjectPool::default();
  let map = slice.map(&pool, &MapOptions::default()).unwrap();
  let first = map.decoded_mappings().next().unwrap();
  assert_eq!((first.generated_line, first.generated_column), (1, 0));
  assert_eq!(first.original.unwrap().original_column, 2);
  assert_eq!(map.sources()[0], "input.js");
  let map = slice
    .boxed()
    .map_static(&pool, &MapOptions::default())
    .unwrap();
  assert_eq!(map.sources_content()[0], "😀你好 world\nnext");
}

#[test]
fn slices_cross_source_chunks_and_can_be_nested() {
  let source = ConcatSource::new([
    OriginalSource::new("ab😀", "a.js").boxed(),
    RawStringSource::from_static("中\ncd").boxed(),
    OriginalSource::new("ef", "b.js").boxed(),
  ])
  .boxed();
  let slice = SourceSlice::new(source.clone(), 2..12).boxed();
  assert_text(slice.as_ref(), "😀中\ncd");
  assert_text(&SourceSlice::new(slice, 4..8), "中\n");
  assert_text(&SourceSlice::new(source.clone(), 0..0), "");
  assert_text(
    &SourceSlice::new(source.clone(), source.size()..source.size()),
    "",
  );
  let slice = SourceSlice::new(source, 2..12);
  for columns in [true, false] {
    let chunks = slice.stream_chunks();
    let mut output = String::new();
    let info = chunks.stream(
      &ObjectPool::default(),
      &MapOptions::new(columns),
      &mut |chunk, _| {
        if let Some(chunk) = chunk {
          output.push_str(chunk.as_str());
        }
      },
      &mut |_, _, _| {},
      &mut |_, _| {},
    );
    assert_eq!(output, "😀中\ncd");
    assert_eq!((info.generated_line, info.generated_column), (2, 2));
  }
}

#[test]
fn replacement_sources_stream_and_survive_cache_roundtrips() {
  use rspack_cacheable::{enable_cacheable as cacheable, from_bytes, to_bytes, with::AsPreset};
  #[cacheable]
  struct Data(#[cacheable(with=AsPreset)] BoxSource);

  let base = OriginalSource::new("hello world!\n", "base.js").boxed();
  let mut source = ReplaceSource::new(base.clone());
  source.replace_source(
    6,
    11,
    SourceSlice::new(base, 0..5).boxed(),
    Some("greeting".into()),
  );
  source.insert_source_with_enforce(
    0,
    RawStringSource::from(Arc::<str>::from("shared ")).boxed(),
    None,
    ReplacementEnforce::Post,
  );
  source.insert_source_with_enforce(
    0,
    RawStringSource::from(SmolStr::new("small ")).boxed(),
    None,
    ReplacementEnforce::Pre,
  );
  let mut nested = ReplaceSource::new(RawStringSource::from_static("tail"));
  nested.replace_static(0, 1, "T", None);
  source.insert_source(100, CachedSource::new(nested).boxed(), None);
  assert_text(&source, "small shared hello hello!\nTail");
  let data = Data(CachedSource::new(source).boxed());
  let bytes = to_bytes(&data, &()).unwrap();
  let restored: Data = from_bytes(&bytes, &()).unwrap();
  assert_eq!(data.0.as_ref(), restored.0.as_ref());
  assert_eq!(hash(data.0.as_ref()), hash(restored.0.as_ref()));
  assert_text(restored.0.as_ref(), "small shared hello hello!\nTail");
  for columns in [true, false] {
    assert_eq!(
      data
        .0
        .map(&ObjectPool::default(), &MapOptions::new(columns)),
      restored
        .0
        .map(&ObjectPool::default(), &MapOptions::new(columns))
    );
  }
}

// A source whose only content API is its writer. Its writes deliberately split
// Unicode scalars, so byte-stream transforms cannot assume each write is UTF-8.
#[derive(Debug, PartialEq, Eq, Hash)]
struct WriterOnly(&'static [u8]);
impl StreamChunks for WriterOnly {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a> {
    panic!("mapping was not requested")
  }
}
impl Source for WriterOnly {
  fn source(&self) -> SourceValue<'_> {
    panic!("must use writer")
  }
  fn rope<'a>(&'a self, _: &mut dyn FnMut(&'a str)) {
    panic!("must use writer")
  }
  fn buffer(&self) -> Cow<'_, [u8]> {
    panic!("must use writer")
  }
  fn size(&self) -> usize {
    self.0.len()
  }
  fn map(&self, _: &ObjectPool, _: &MapOptions) -> Option<SourceMap<'_>> {
    None
  }
  fn map_static(self: Arc<Self>, _: &ObjectPool, _: &MapOptions) -> Option<SourceMap<'static>> {
    None
  }
  fn to_writer(&self, writer: &mut dyn Write) -> io::Result<()> {
    for byte in self.0 {
      writer.write_all(&[*byte])?;
    }
    Ok(())
  }
}

#[test]
fn writer_path_never_materializes_or_requests_rope() {
  let mut source = ReplaceSource::new(WriterOnly("a😀bcdef".as_bytes()));
  source.replace_source(
    5,
    7,
    SourceSlice::new(WriterOnly(b"--XYZ--"), 2..5).boxed(),
    None,
  );
  source.replace_source(6, 100, WriterOnly(b"!").boxed(), None);
  source.insert_source(1000, WriterOnly(b"tail").boxed(), None);
  let mut output = Vec::new();
  source.to_writer(&mut output).unwrap();
  assert_eq!(output, "a😀XYZ!tail".as_bytes());
  assert_eq!(source.size(), output.len());
  assert_eq!(source.buffer().as_ref(), output);
  assert_eq!(source.source().into_string_lossy(), "a😀XYZ!tail");
  let unchanged = ReplaceSource::new(WriterOnly(b"untouched"));
  let mut output = Vec::new();
  unchanged.to_writer(&mut output).unwrap();
  assert_eq!(output, b"untouched");
}

#[test]
fn writer_errors_and_binary_bytes_are_preserved() {
  struct FailingWriter;
  impl Write for FailingWriter {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
      Err(io::ErrorKind::BrokenPipe.into())
    }
    fn flush(&mut self) -> io::Result<()> {
      Ok(())
    }
  }
  let mut source = ReplaceSource::new(RawBufferSource::from(vec![0xff, 0, 1]));
  source.replace_source(1, 2, RawBufferSource::from(vec![0xfe]).boxed(), None);
  assert_eq!(source.buffer().as_ref(), &[0xff, 0xfe, 1]);
  assert_eq!(source.size(), 3);
  assert_eq!(
    source.to_writer(&mut FailingWriter).unwrap_err().kind(),
    io::ErrorKind::BrokenPipe
  );
  let mut insertion = ReplaceSource::new(RawStringSource::from_static(""));
  insertion.insert_source(10, WriterOnly(b"replacement").boxed(), None);
  assert_eq!(
    insertion.to_writer(&mut FailingWriter).unwrap_err().kind(),
    io::ErrorKind::BrokenPipe
  );
  assert_eq!(
    SourceSlice::new(source.boxed(), 1..3).buffer().as_ref(),
    &[0xfe, 1]
  );
}

#[test]
fn every_slice_boundary_matches_flat_text_and_streamed_positions() {
  let text = "a😀\n中bc\nend";
  let source = ConcatSource::new([
    OriginalSource::new("a😀\n", "a.js"),
    OriginalSource::new("中bc\n", "b.js"),
    OriginalSource::new("end", "c.js"),
  ])
  .boxed();
  let boundaries: Vec<_> = (0..=text.len())
    .filter(|&i| text.is_char_boundary(i))
    .collect();
  for &start in &boundaries {
    for &end in boundaries.iter().filter(|&&end| end >= start) {
      let slice = SourceSlice::new(source.clone(), start..end);
      let expected = &text[start..end];
      assert_text(&slice, expected);
      for columns in [true, false] {
        let mut rendered = String::new();
        let chunks = slice.stream_chunks();
        let info = chunks.stream(
          &ObjectPool::default(),
          &MapOptions::new(columns),
          &mut |chunk, _| {
            if let Some(chunk) = chunk {
              rendered.push_str(chunk.as_str());
            }
          },
          &mut |_, _, _| {},
          &mut |_, _| {},
        );
        assert_eq!(rendered, expected, "{start}..{end}");
        assert_eq!(
          info.generated_line as usize,
          expected.bytes().filter(|&b| b == b'\n').count() + 1,
          "{start}..{end}"
        );
        assert_eq!(
          info.generated_column as usize,
          expected.rsplit('\n').next().unwrap().encode_utf16().count(),
          "{start}..{end}"
        );
      }
    }
  }
}

#[test]
fn fragmented_replacement_content_keeps_names_and_unicode_positions() {
  let mut source = ReplaceSource::new(OriginalSource::new("start old end", "base.js"));
  source.replace_source(
    6,
    9,
    ConcatSource::new([
      OriginalSource::new("😀", "ignored-a.js"),
      OriginalSource::new("中\nx", "ignored-b.js"),
    ])
    .boxed(),
    Some("replacement".into()),
  );
  assert_text(&source, "start 😀中\nx end");
  let pool = ObjectPool::default();
  let map = source.map(&pool, &MapOptions::default()).unwrap();
  assert_eq!(map.sources(), &[Cow::Borrowed("base.js")]);
  assert_eq!(map.names(), &[Cow::Borrowed("replacement")]);
  let mappings: Vec<_> = map.decoded_mappings().collect();
  assert!(mappings.iter().any(|m| m.generated_line == 1
    && m.generated_column == 6
    && m.original.as_ref().is_some_and(|o| o.name_index == Some(0))));
  assert!(mappings.iter().any(|m| m.generated_line == 2
    && m.generated_column == 1
    && m.original.as_ref().is_some_and(|o| o.original_column == 9)));
}
