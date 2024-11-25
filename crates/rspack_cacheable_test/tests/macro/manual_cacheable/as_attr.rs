use rspack_cacheable::{
  from_bytes, to_bytes,
  with::{AsTuple2, Inline},
};

#[derive(
  rspack_cacheable::__private::rkyv::Archive,
  rspack_cacheable::__private::rkyv::Deserialize,
  rspack_cacheable::__private::rkyv::Serialize,
)]
#[rkyv(crate=rspack_cacheable::__private::rkyv)]
#[derive(Debug, PartialEq, Eq)]
struct Person {
  name: String,
  content: (String, String),
}

#[derive(
  rspack_cacheable::__private::rkyv::Archive, rspack_cacheable::__private::rkyv::Serialize,
)]
#[rkyv(crate=rspack_cacheable::__private::rkyv,
         as=rspack_cacheable::__private::rkyv::Archived<Person>)]
#[rkyv(serialize_bounds(
    __S: rspack_cacheable::__private::rkyv::ser::Writer + rspack_cacheable::__private::rkyv::ser::Allocator + rspack_cacheable::__private::rkyv::rancor::Fallible<Error = rspack_cacheable::SerializeError>,
  ))]
#[derive(Debug, PartialEq, Eq)]
struct PersonRef<'a> {
  #[rkyv(omit_bounds)]
  #[rkyv(with=Inline)]
  name: &'a String,
  #[rkyv(omit_bounds)]
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
