use rspack_cacheable::{cacheable, from_bytes, to_bytes};

#[cacheable]
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
