use rspack_cacheable::{enable_cacheable as cacheable, from_bytes, to_bytes, with::AsPreset};
use rspack_sources::{
  BoxSource, ObjectPool, RawBufferSource, RawStringSource, SourceExt, SourceMap, SourceMapSource,
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
    assert_eq!(&data.0, &new_data.0);
    let mut written = Vec::new();
    new_data.0.to_writer(&mut written).unwrap();
    assert_eq!(written.as_slice(), new_data.0.buffer().as_ref());
    assert_eq!(new_data.0.size(), written.len());
    assert_eq!(
      data.0.map(&ObjectPool::default(), &Default::default()),
      new_data.0.map(&ObjectPool::default(), &Default::default())
    );
  }

  test_data(Data(RawBufferSource::from("123".as_bytes()).boxed()));
  test_data(Data(RawStringSource::from_static("123").boxed()));
  let buffer = vec![0, 255, 254, 128, 10];
  let source = SourceMapSource::from_buffer(
    buffer.clone(),
    "binary.js",
    SourceMap::from_json(r#"{"version":3,"sources":["original.js"],"sourcesContent":["original"],"names":[],"mappings":"AAAA"}"#.to_string()).unwrap(),
  ).boxed();
  assert_eq!(source.buffer().as_ref(), buffer.as_slice());
  test_data(Data(source));
}

#[test]
fn consuming_sources_reuses_unique_allocations_before_and_after_cache() {
  fn check(source: BoxSource) {
    let expected = source.buffer().into_owned();
    let pointer = source.buffer().as_ptr();
    let value = source.into_source_value();
    assert_eq!(value.as_bytes(), expected);
    assert_eq!(value.as_bytes().as_ptr(), pointer);
  }
  let map = || {
    SourceMap::from_json(
      r#"{"version":3,"sources":["input.js"],"names":[],"mappings":"AAAA"}"#.to_string(),
    )
    .unwrap()
  };
  let sources = [
    RawBufferSource::from("héllo".as_bytes().to_vec()).boxed(),
    RawStringSource::from("héllo".to_owned()).boxed(),
    rspack_sources::OriginalSource::new("héllo", "input.js").boxed(),
    SourceMapSource::from_buffer(b"hello".to_vec(), "input.js", map()).boxed(),
    SourceMapSource::from_buffer(vec![0xff, 0, 0xfe], "input.js", map()).boxed(),
  ];
  for source in sources {
    let data = Data(source);
    let serialized = to_bytes(&data, &()).unwrap();
    let restored: Data = from_bytes(&serialized, &()).unwrap();
    check(data.0);
    check(restored.0);
  }
}

#[test]
fn consuming_shared_sources_preserves_the_other_owner() {
  for source in [
    RawBufferSource::from(vec![0xff, 0, 0xfe]).boxed(),
    RawStringSource::from("shared text".to_owned()).boxed(),
  ] {
    let expected = source.buffer().into_owned();
    let value = source.clone().into_source_value();
    assert_eq!(value.as_bytes(), expected);
    assert_eq!(source.buffer().as_ref(), expected);
  }
  for bytes in [Vec::new(), "héllo".as_bytes().to_vec(), vec![0xff, 0, 0xfe]] {
    let expected = String::from_utf8_lossy(&bytes).into_owned();
    let value = RawBufferSource::from(bytes)
      .boxed()
      .into_source_value()
      .into_string_lossy();
    assert_eq!(value, expected);
  }
}
