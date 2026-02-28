use std::path::PathBuf;

use rspack_cacheable::{
  enable_cacheable as cacheable,
  utils::PortablePath,
  with::{As, AsCacheable, AsTuple3},
};

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Data {
  param1: (u32, u32, u32),
  #[cacheable(with=AsTuple3)]
  param2: (u32, u32, u32),
  #[cacheable(with=AsTuple3<AsCacheable, AsCacheable, As<PortablePath>>)]
  param3: (u32, u32, PathBuf),
}

#[test]
fn test_as_tuple3() {
  let data = Data {
    param1: (1, 2, 3),
    param2: (4, 5, 6),
    param3: (7, 8, PathBuf::from("/root")),
  };

  let bytes = rspack_cacheable::to_bytes(&data, &()).unwrap();
  let new_data: Data = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(data, new_data);
}
