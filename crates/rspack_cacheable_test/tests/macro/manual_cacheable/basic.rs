use rspack_cacheable::{from_bytes, to_bytes};

#[derive(
  rspack_cacheable::__private::rkyv::Archive,
  rspack_cacheable::__private::rkyv::Deserialize,
  rspack_cacheable::__private::rkyv::Serialize,
)]
#[rkyv(crate=rspack_cacheable::__private::rkyv)]
#[derive(Debug, PartialEq, Eq)]
struct Person {
  name: String,
}

#[test]
fn basic_macro_feature() {
  let a = Person {
    name: String::from("a"),
  };
  let bytes = to_bytes(&a, &()).unwrap();
  let deserialize_a = from_bytes(&bytes, &()).unwrap();
  assert_eq!(a, deserialize_a);
}
