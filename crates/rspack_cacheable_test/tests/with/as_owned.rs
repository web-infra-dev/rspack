use std::borrow::Cow;

use rspack_cacheable::{enable_cacheable as cacheable, utils::OwnedOrRef, with::AsOwned};

#[cacheable]
#[derive(Debug, PartialEq, Eq, Clone)]
struct Param(String);

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Data<'a> {
  #[cacheable(with=AsOwned)]
  param1: Cow<'a, Param>,
  param2: Param,
  #[cacheable(with=AsOwned)]
  param3: OwnedOrRef<'a, Param>,
}

#[test]
fn test_as_owned() {
  let param1 = Param(String::from("param1"));
  let data = Data {
    param1: Cow::Borrowed(&param1),
    param2: Param(String::from("param2")),
    param3: OwnedOrRef::Borrowed(&param1),
  };

  assert!(matches!(data.param1, Cow::Borrowed(_)));
  assert!(matches!(data.param3, OwnedOrRef::Borrowed(_)));
  let bytes = rspack_cacheable::to_bytes(&data, &()).unwrap();
  let new_data: Data = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(data, new_data);
  assert!(matches!(new_data.param1, Cow::Owned(_)));
  assert!(matches!(new_data.param3, OwnedOrRef::Owned(_)));
}
