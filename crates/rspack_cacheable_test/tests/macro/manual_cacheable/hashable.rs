use rspack_cacheable::{from_bytes, to_bytes};
use rustc_hash::FxHashSet as HashSet;

#[derive(
  rspack_cacheable::__private::rkyv::Archive,
  rspack_cacheable::__private::rkyv::Deserialize,
  rspack_cacheable::__private::rkyv::Serialize,
)]
#[rkyv(crate=rspack_cacheable::__private::rkyv)]
#[rkyv(derive(Hash, PartialEq, Eq))]
#[derive(Debug, Hash, PartialEq, Eq)]
struct Person {
  name: String,
}

#[test]
fn hashable_attr() {
  let mut a = HashSet::default();
  a.insert(Person {
    name: String::from("a"),
  });
  let bytes = to_bytes(&a, &()).unwrap();
  let deserialize_a: HashSet<Person> = from_bytes(&bytes, &()).unwrap();
  assert_eq!(a, deserialize_a);
}
