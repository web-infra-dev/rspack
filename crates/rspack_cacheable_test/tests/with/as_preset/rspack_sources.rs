use rspack_cacheable::{enable_cacheable as cacheable, from_bytes, to_bytes, with::AsPreset};
use rspack_sources::{
  BoxSource, RawBufferSource, RawStringSource, SourceExt, SourceMap, SourceMapSource,
  WithoutOriginalOptions, cacheable::SourceContentIndices,
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
  test_data(Data(
    SourceMapSource::new(WithoutOriginalOptions {
      value: "generated",
      name: "out.js",
      source_map: SourceMap::from_json(
        r#"{"version":3,"sources":["a"],"sourcesContent":[null],"names":[],"mappings":"AAAA"}"#
          .into(),
      )
      .unwrap(),
    })
    .boxed(),
  ));
}

#[test]
fn source_content_indices_round_trip() {
  let map = SourceMap::from_json(
    r#"{"version":3,"sources":["a","b","c"],"sourcesContent":[null,"","original"],"names":[],"mappings":"AAAA"}"#.into(),
  )
  .unwrap();
  let original = SourceMapSource::new(WithoutOriginalOptions {
    value: "generated",
    name: "out.js",
    source_map: map.clone(),
  });
  let indices = SourceContentIndices::new(&original, &map).unwrap();
  let bytes = to_bytes(&Data(original.boxed()), &()).unwrap();
  let restored: Data = from_bytes(&bytes, &()).unwrap();
  let restored_map = restored.0.map(&Default::default()).unwrap();
  let contents = indices.restore(restored.0.as_ref()).unwrap();

  assert_eq!(contents, map.sources_content());
  assert_eq!(
    contents[2].as_ref().unwrap().as_ptr(),
    restored_map.get_source_content(2).unwrap().as_ptr(),
  );
  assert!(
    indices
      .restore(&RawStringSource::from_static("generated"))
      .is_none()
  );
}
