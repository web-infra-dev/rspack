use rspack_cacheable::{enable_cacheable as cacheable, from_bytes, to_bytes, with::AsPreset};
use rspack_sources::{BoxSource, RawBufferSource, RawStringSource, SourceExt};

#[cacheable]
#[derive(Debug)]
struct Data(#[cacheable(with=AsPreset)] BoxSource);

#[test]
fn test_rspack_source() {
  fn test_data(data: Data) {
    let bytes = to_bytes(&data, &()).unwrap();
    let new_data: Data = from_bytes(&bytes, &()).unwrap();
    assert_eq!(data.0.buffer(), new_data.0.buffer());
    assert_eq!(
      data.0.map(&Default::default()),
      new_data.0.map(&Default::default())
    );
  }

  test_data(Data(RawBufferSource::from("123".as_bytes()).boxed()));
  test_data(Data(RawStringSource::from_static("123").boxed()));
}

#[test]
fn dev_tool_code_keeps_graph_and_cache_shape() {
  use std::sync::Arc;

  use rspack_sources::{ConcatSource, DevToolCodeSource, OriginalSource, Source};

  let inner = OriginalSource::new("console.log('你好');", "input.js").boxed();
  let suffix = "\n//# sourceMappingURL=out.js.map";
  let source = DevToolCodeSource::new(Arc::clone(&inner), RawStringSource::from(suffix).boxed());
  assert!(Arc::ptr_eq(source.inner(), &inner));
  assert!(source.map(&Default::default()).is_none());
  let expected = format!("console.log('你好');{suffix}");
  assert_eq!(source.buffer().as_ref(), expected.as_bytes());
  assert_eq!(source.size(), expected.len());
  let mut rope = String::new();
  source.rope(&mut |chunk| rope.push_str(chunk));
  assert_eq!(rope, expected);
  let mut output = Vec::new();
  source.to_writer(&mut output).unwrap();
  assert_eq!(output, expected.as_bytes());

  for suffix in ["", suffix] {
    let data = Data(
      DevToolCodeSource::new(Arc::clone(&inner), RawStringSource::from(suffix).boxed()).boxed(),
    );
    let bytes = to_bytes(&data, &()).unwrap();
    let restored: Data = from_bytes(&bytes, &()).unwrap();
    let code = restored
      .0
      .as_any()
      .downcast_ref::<DevToolCodeSource>()
      .unwrap();
    assert_eq!(code.suffix().source().into_string_lossy(), suffix);
    assert!(code.inner().as_any().is::<OriginalSource>());
    assert_eq!(data.0.buffer(), restored.0.buffer());
    assert!(restored.0.map(&Default::default()).is_none());
    // Wrapping the result must not reveal the original mappings via stream_chunks.
    let concat = ConcatSource::new([restored.0, RawStringSource::from("\n").boxed()]);
    assert!(concat.map(&Default::default()).is_none());
  }
}

#[test]
fn dev_tool_code_preserves_binary_normalization() {
  use rspack_sources::{ConcatSource, DevToolCodeSource, Source};

  let bytes = vec![0xff, 0, b'x'];
  let bare = DevToolCodeSource::new(
    RawBufferSource::from(bytes.clone()).boxed(),
    RawStringSource::from_static("").boxed(),
  );
  assert!(!bare.is_text());
  assert_eq!(bare.buffer().as_ref(), bytes);
  let appended = DevToolCodeSource::new(
    RawBufferSource::from(bytes.clone()).boxed(),
    RawStringSource::from_static("!").boxed(),
  );
  assert!(!appended.is_text());
  assert_eq!(appended.buffer().as_ref(), [0xff, 0, b'x', b'!']);
  assert_eq!(appended.source().into_string_lossy(), "�\0x!");

  let mixed = ConcatSource::new([
    RawBufferSource::from(bytes).boxed(),
    RawStringSource::from("text").boxed(),
  ])
  .boxed();
  let expected = format!("{}!", mixed.source().into_string_lossy());
  let normalized = DevToolCodeSource::new(mixed, RawStringSource::from_static("!").boxed());
  assert!(normalized.is_text());
  assert_eq!(normalized.buffer().as_ref(), expected.as_bytes());
}

#[test]
fn source_map_sources_compare_rendered_bytes() {
  use rspack_sources::{
    DevToolCodeSource, DevToolMapSource, OriginalSource, Source, SourceMap, source_content_eq,
  };

  let map = SourceMap::from_json(r#"{"version":3,"sources":["a\"b.js"],"sourcesContent":["\u0000\n你好\\world"],"names":[],"mappings":"AAAA"}"#.to_owned()).unwrap();
  let source = DevToolMapSource::new(map);
  let text = source.source().into_string_lossy().into_owned();
  let split = text.find("sourcesContent").unwrap();
  let code = DevToolCodeSource::new(
    OriginalSource::new(text[..split].to_owned(), "prefix").boxed(),
    RawStringSource::from(text[split..].to_owned()).boxed(),
  );
  let raw = RawStringSource::from(text.clone());
  assert!(source_content_eq(&source, &code));
  assert!(source_content_eq(&code, &source));
  assert!(source_content_eq(&code, &raw));
  assert!(!source_content_eq(
    &code,
    &RawStringSource::from(format!("{text}x"))
  ));
  let mut rope = String::new();
  source.rope(&mut |chunk| rope.push_str(chunk));
  assert_eq!(rope, text);
  let data = Data(source.boxed());
  let bytes = to_bytes(&data, &()).unwrap();
  let restored: Data = from_bytes(&bytes, &()).unwrap();
  assert!(restored.0.as_any().is::<DevToolMapSource>());
  assert_eq!(restored.0.buffer(), data.0.buffer());
}

#[test]
fn source_map_edits_share_graph_and_survive_cache() {
  use std::sync::Arc;

  use rspack_sources::{
    DevToolCodeSource, DevToolMapSource, OriginalSource, ReplaceSource, Source, SourceMap,
    is_source_map_text, source_content_eq,
  };

  let inner = OriginalSource::new("你好;body", "input.js").boxed();
  let suffix = "\n//# sourceMappingURL=out.deadbeef.js.map";
  let source =
    DevToolCodeSource::new(Arc::clone(&inner), RawStringSource::from(suffix).boxed()).boxed();
  let start = inner.size() + suffix.find("deadbeef").unwrap();
  for (from, to, replacement, expected) in [
    (
      start,
      start + 8,
      "newhash",
      "你好;body\n//# sourceMappingURL=out.newhash.js.map".to_owned(),
    ),
    (
      start,
      start + 8,
      "",
      "你好;body\n//# sourceMappingURL=out..js.map".to_owned(),
    ),
    (
      "你好;".len(),
      inner.size(),
      "changed",
      format!("你好;changed{suffix}"),
    ),
    (
      inner.size() - 1,
      inner.size() + 1,
      "!",
      "你好;bod!//# sourceMappingURL=out.deadbeef.js.map".to_owned(),
    ),
  ] {
    let mut edited = ReplaceSource::new(Arc::clone(&source));
    edited.replace(from as u32, to as u32, replacement.to_owned(), None);
    assert!(Arc::ptr_eq(edited.inner(), &source));
    assert!(Arc::ptr_eq(
      source
        .as_any()
        .downcast_ref::<DevToolCodeSource>()
        .unwrap()
        .inner(),
      &inner
    ));
    assert!(is_source_map_text(&edited));
    assert!(edited.map(&Default::default()).is_none());
    assert!(source_content_eq(&edited, &RawStringSource::from(expected)));
    let data = Data(edited.boxed());
    let restored: Data = from_bytes(&to_bytes(&data, &()).unwrap(), &()).unwrap();
    assert!(restored.0.as_any().is::<ReplaceSource>());
    assert!(is_source_map_text(restored.0.as_ref()));
    assert!(data.0.as_ref() == restored.0.as_ref());
    assert_eq!(data.0.buffer(), restored.0.buffer());
    assert!(restored.0.map(&Default::default()).is_none());
  }

  let map = SourceMap::from_json(r#"{"version":3,"file":"deadbeef.js","sources":["a.js"],"sourcesContent":["😀\n\"deadbeef\""],"names":[],"mappings":"AAAA"}"#.to_owned()).unwrap();
  let source = DevToolMapSource::new(map).boxed();
  let text = source.source().into_string_lossy().into_owned();
  let mut edited = ReplaceSource::new(Arc::clone(&source));
  for (start, _) in text.match_indices("deadbeef") {
    edited.replace(start as u32, (start + 8) as u32, "new".to_owned(), None);
  }
  assert!(Arc::ptr_eq(edited.inner(), &source));
  assert!(source_content_eq(
    &edited,
    &RawStringSource::from(text.split("deadbeef").collect::<Vec<_>>().join("new"))
  ));
  assert!(edited.map(&Default::default()).is_none());
  let data = Data(edited.boxed());
  let restored: Data = from_bytes(&to_bytes(&data, &()).unwrap(), &()).unwrap();
  assert!(data.0.as_ref() == restored.0.as_ref());
  assert!(source_content_eq(data.0.as_ref(), restored.0.as_ref()));
  assert!(restored.0.map(&Default::default()).is_none());
}

#[test]
fn replaced_content_cursor_matches_rope_and_hash() {
  use std::hash::{DefaultHasher, Hasher};

  use rspack_sources::{
    ConcatSource, DevToolCodeSource, ReplaceSource, Source, source_content_eq,
    update_text_content_hash,
  };
  let source = DevToolCodeSource::new(
    ConcatSource::new([
      RawStringSource::from_static("ab").boxed(),
      RawStringSource::from_static("cdef").boxed(),
    ])
    .boxed(),
    RawStringSource::from_static("gh").boxed(),
  )
  .boxed();
  // Include overlapping ranges, end/beyond-end insertions, a cross-fragment
  // edit, empty replacement strings and empty generated content.
  for edits in [
    vec![],
    vec![(0, 8, "")],
    vec![(1, 7, "你好")],
    vec![(1, 4, "X"), (2, 6, "Y"), (6, 6, "!")],
    vec![(1, 20, "X"), (2, 3, "Y"), (30, 30, "Z")],
    vec![(8, 8, "!"), (9, 10, "?")],
  ] {
    let mut source = ReplaceSource::new(source.clone());
    for (start, end, text) in edits {
      source.replace(start, end, text.to_owned(), None);
    }
    let raw = RawStringSource::from(source.source().into_string_lossy().into_owned());
    assert!(
      source_content_eq(&source, &raw),
      "edits={:?}, size={}, expected={:?}",
      source.replacements(),
      source.size(),
      raw.source()
    );
    assert!(source_content_eq(&raw, &source));
    assert!(!source_content_eq(
      &source,
      &RawStringSource::from(format!("{}!", raw.source().into_string_lossy()))
    ));
    let mut expected = DefaultHasher::new();
    expected.write(&raw.buffer());
    let mut actual = DefaultHasher::new();
    update_text_content_hash(&source, &mut actual);
    assert_eq!(actual.finish(), expected.finish());
    let mut nested = ReplaceSource::new(source.boxed());
    nested.insert(0, "前".to_owned(), None);
    let raw = RawStringSource::from(nested.source().into_string_lossy().into_owned());
    assert!(source_content_eq(&nested, &raw));
    // Same-length mismatch must compare bytes, not only generated lengths.
    let wrong = RawStringSource::from(format!(
      "后{}",
      &raw.source().into_string_lossy()["前".len()..]
    ));
    assert!(!source_content_eq(&nested, &wrong));
  }
}
