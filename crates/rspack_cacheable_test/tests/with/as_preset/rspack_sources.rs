use rspack_cacheable::{enable_cacheable as cacheable, from_bytes, to_bytes, with::AsPreset};
use rspack_sources::{
  BoxSource, DevToolMapSource, RawBufferSource, RawStringSource, SourceExt, SourceMap,
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
    assert_eq!(
      data.0.map(&Default::default()),
      new_data.0.map(&Default::default())
    );
  }

  test_data(Data(RawBufferSource::from("123".as_bytes()).boxed()));
  test_data(Data(RawStringSource::from_static("123").boxed()));
}

#[test]
fn test_devtool_map_source() {
  let map = SourceMap::from_json(
    r#"{
      "version": 3,
      "file": "bundle.js",
      "sources": ["missing.js", "empty.js", "源.js"],
      "sourcesContent": [null, "", "const message = '你好';"],
      "names": ["消息"],
      "mappings": "AAEAA",
      "sourceRoot": "/src",
      "debugId": "",
      "ignoreList": []
    }"#
      .to_owned(),
  )
  .unwrap();
  let data = Data(DevToolMapSource::new(map).boxed());
  let mut bytes = to_bytes(&data, &()).unwrap();
  let restored: Data = from_bytes(&bytes, &()).unwrap();
  assert_eq!(data.0.as_ref(), restored.0.as_ref());
  assert_eq!(data.0.buffer(), restored.0.buffer());

  let content = "const message = '你好';".as_bytes();
  let offset = bytes
    .windows(content.len())
    .position(|window| window == content)
    .unwrap();
  bytes[offset] = 0xff;
  assert!(from_bytes::<Data, _>(&bytes, &()).is_err());
}
