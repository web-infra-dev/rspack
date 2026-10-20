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
