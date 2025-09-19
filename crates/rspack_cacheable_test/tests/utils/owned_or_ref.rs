use rspack_cacheable::{cacheable, utils::OwnedOrRef};

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Data(String);

#[test]
fn test_owned_or_ref() {
  let data = Data("abc".into());
  let bytes = rspack_cacheable::to_bytes(&OwnedOrRef::Borrowed(&data), &()).unwrap();
  let data_ref: OwnedOrRef<'static, Data> = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(data, data_ref.into_owned());
}
