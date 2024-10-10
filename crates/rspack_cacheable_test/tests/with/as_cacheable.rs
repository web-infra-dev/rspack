use rspack_cacheable::{cacheable, with::AsCacheable};

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Param(String);

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Data {
  #[cacheable(with=AsCacheable)]
  param1: Param,
  param2: Param,
}

#[test]
fn test_as_cacheable() {
  let data = Data {
    param1: Param(String::from("param1")),
    param2: Param(String::from("param2")),
  };

  let bytes = rspack_cacheable::to_bytes(&data, &()).unwrap();
  let new_data: Data = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(data, new_data);
}
