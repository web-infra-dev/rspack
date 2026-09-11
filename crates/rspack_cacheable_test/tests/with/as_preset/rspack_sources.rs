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
