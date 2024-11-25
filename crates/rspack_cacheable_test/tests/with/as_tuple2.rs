use std::path::PathBuf;

use rspack_cacheable::{
  cacheable,
  with::{AsCacheable, AsString, AsTuple2},
};

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Data {
  param1: (u32, u32),
  #[cacheable(with=AsTuple2)]
  param2: (u32, u32),
  #[cacheable(with=AsTuple2<AsCacheable, AsString>)]
  param3: (u32, PathBuf),
}

#[test]
fn test_as_tuple2() {
  let data = Data {
    param1: (1, 2),
    param2: (3, 4),
    param3: (5, PathBuf::from("/root")),
  };

  let bytes = rspack_cacheable::to_bytes(&data, &()).unwrap();
  let new_data: Data = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(data, new_data);
}
