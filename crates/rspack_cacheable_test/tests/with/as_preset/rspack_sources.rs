use rspack_cacheable::{enable_cacheable as cacheable, from_bytes, to_bytes, with::AsPreset};
use rspack_sources::{
  BoxSource, OriginalSource, RawBufferSource, RawStringSource, SourceExt, SourceOnlySource,
};

#[cacheable]
#[derive(Debug)]
struct Data(#[cacheable(with=AsPreset)] BoxSource);

#[test]
fn test_rspack_source() {
  fn test_data(data: Data) {
    let bytes = to_bytes(&data, &()).unwrap();
    let new_data: Data = from_bytes(&bytes, &()).unwrap();
    assert_eq!(data.0.buffer(), new_data.0.buffer());
    assert!(data.0.as_ref().eq(new_data.0.as_ref()));
    assert_eq!(
      data.0.map(&Default::default()),
      new_data.0.map(&Default::default())
    );
  }

  test_data(Data(RawBufferSource::from("123".as_bytes()).boxed()));
  test_data(Data(RawStringSource::from_static("123").boxed()));
  test_data(Data(
    SourceOnlySource::new(OriginalSource::new("hello 😀", "original.js").boxed()).boxed(),
  ));
  test_data(Data(
    SourceOnlySource::new(RawBufferSource::from(vec![0xff, 0, 0x61]).boxed()).boxed(),
  ));
}

#[test]
fn source_only_preserves_materialized_bytes() {
  use std::sync::Arc;

  use rspack_sources::{CachedSource, ConcatSource, MapOptions, ReplaceSource, SourceValue};

  let binary = RawBufferSource::from(vec![0xff, 0, b'x']).boxed();
  let mixed = ConcatSource::new([
    OriginalSource::new("// mapped\n", "original.js").boxed(),
    Arc::clone(&binary),
  ])
  .boxed();
  assert!(mixed.map(&MapOptions::default()).is_some());

  let sources = [
    Arc::clone(&binary),
    Arc::clone(&mixed),
    CachedSource::new(Arc::clone(&mixed)).boxed(),
    ReplaceSource::new(Arc::clone(&mixed)).boxed(),
    ReplaceSource::new(Arc::clone(&binary)).boxed(),
    ConcatSource::new([Arc::clone(&binary)]).boxed(),
  ];
  for inner in sources {
    let expected = inner.source();
    let is_binary = matches!(expected, SourceValue::Buffer(_));
    let expected = expected.into_bytes().into_owned();
    let source = SourceOnlySource::new(inner).boxed();
    let bytes = to_bytes(&Data(Arc::clone(&source)), &()).unwrap();
    let restored: Data = from_bytes(&bytes, &()).unwrap();
    for source in [source, restored.0] {
      assert_eq!(matches!(source.source(), SourceValue::Buffer(_)), is_binary);
      assert_eq!(source.source().into_bytes().as_ref(), expected);
      assert_eq!(source.buffer().as_ref(), expected);
      let mut written = Vec::new();
      source.to_writer(&mut written).unwrap();
      assert_eq!(written, expected);
      assert_eq!(source.size(), expected.len());
      assert!(source.map(&MapOptions::default()).is_none());
    }
  }

  // Ordinary UTF-8 graphs stay shared rather than being flattened.
  let text = ConcatSource::new([
    OriginalSource::new("// mapped\n", "original.js").boxed(),
    RawBufferSource::from("😀".as_bytes()).boxed(),
  ])
  .boxed();
  let source = SourceOnlySource::new(Arc::clone(&text));
  assert!(Arc::ptr_eq(source.inner(), &text));
}
