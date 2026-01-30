use rspack_cacheable::{enable_cacheable as cacheable, from_bytes, to_bytes, with::AsPreset};
use rspack_sources::{BoxSource, ObjectPool, RawBufferSource, RawStringSource, SourceExt};

#[cacheable]
#[derive(Debug)]
struct Data(#[cacheable(with=AsPreset)] BoxSource);

#[test]
fn test_rspack_source() {
  fn test_data(data: &Data) {
    let bytes = to_bytes(data, &()).unwrap();
    let new_data: Data = from_bytes(&bytes, &()).unwrap();
    assert_eq!(data.0.buffer(), new_data.0.buffer());
    assert_eq!(
      data.0.map(&ObjectPool::default(), &Default::default()),
      new_data.0.map(&ObjectPool::default(), &Default::default())
    );
  }

  test_data(&Data(RawBufferSource::from("123".as_bytes()).boxed()));
  test_data(&Data(RawStringSource::from_static("123").boxed()));
}
