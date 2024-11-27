use rspack_cacheable::{
  cacheable, from_bytes, to_bytes,
  with::{AsRefStr, AsRefStrConverter},
};

#[derive(Debug, PartialEq, Eq)]
struct UnCacheable;

#[cacheable(with=AsRefStr)]
#[derive(Debug, PartialEq, Eq)]
struct Person {
  name: String,
  uncacheable: UnCacheable,
}

impl AsRefStrConverter for Person {
  fn as_str(&self) -> &str {
    &self.name
  }
  fn from_str(s: &str) -> Self {
    Self {
      name: String::from(s),
      uncacheable: UnCacheable,
    }
  }
}

#[test]
fn with_attr() {
  let a = Person {
    name: String::from("a"),
    uncacheable: UnCacheable,
  };
  let bytes = to_bytes(&a, &()).unwrap();
  let deserialize_a = from_bytes(&bytes, &()).unwrap();
  assert_eq!(a, deserialize_a);
}
