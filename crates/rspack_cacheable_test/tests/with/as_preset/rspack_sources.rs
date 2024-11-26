use rspack_cacheable::{cacheable, from_bytes, to_bytes, with::AsPreset};
use rspack_sources::{BoxSource, RawSource, SourceExt};

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

  test_data(Data(RawSource::from("123".as_bytes()).boxed()));
  test_data(Data(RawSource::from("123").boxed()));
}
