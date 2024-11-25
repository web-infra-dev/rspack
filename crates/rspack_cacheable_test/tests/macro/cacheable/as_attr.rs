use rspack_cacheable::{
  cacheable, from_bytes, to_bytes,
  with::{AsTuple2, Inline},
};

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Person {
  name: String,
  content: (String, String),
}

#[cacheable(as=Person)]
struct PersonRef<'a> {
  #[rkyv(with=Inline)]
  name: &'a String,
  #[rkyv(with=AsTuple2<Inline, Inline>)]
  content: (&'a String, &'a String),
}

#[test]
fn as_attr() {
  let a = Person {
    name: "abc".into(),
    content: ("a".into(), "b".into()),
  };
  let a_ref = PersonRef {
    name: &a.name,
    content: (&a.content.0, &a.content.1),
  };
  let bytes = to_bytes(&a_ref, &()).unwrap();
  let deserialize_a: Person = from_bytes(&bytes, &()).unwrap();
  assert_eq!(a, deserialize_a);
}
