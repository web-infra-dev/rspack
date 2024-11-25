use rspack_cacheable::{
  cacheable,
  with::{AsCacheable, AsTuple2, Inline},
};

#[cacheable]
struct Data {
  block1: String,
  block2: (String, String),
}

#[cacheable]
struct DataRef<'a> {
  #[cacheable(with=Inline)]
  block1: &'a String,
  #[cacheable(with=AsTuple2<AsCacheable, Inline>)]
  block2: (String, &'a String),
}

#[test]
fn test_inline() {
  let data = Data {
    block1: "block1".into(),
    block2: ("block2_key".into(), "block2_value".into()),
  };
  let bytes = rspack_cacheable::to_bytes(&data, &()).unwrap();

  let data_ref = DataRef {
    block1: &data.block1,
    block2: (data.block2.0.clone(), &data.block2.1),
  };
  let bytes_ref = rspack_cacheable::to_bytes(&data_ref, &()).unwrap();
  assert_eq!(bytes, bytes_ref);
}
