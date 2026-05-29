use std::collections::BTreeSet;

use rspack_cacheable::{enable_cacheable as cacheable, from_bytes, to_bytes};

#[cacheable(orderable)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Person {
  name: String,
}

#[test]
fn orderable_attr() {
  let mut a = BTreeSet::default();
  a.insert(Person {
    name: String::from("a"),
  });
  let bytes = to_bytes(&a, &()).unwrap();
  let deserialize_a: BTreeSet<Person> = from_bytes(&bytes, &()).unwrap();
  assert_eq!(a, deserialize_a);
}
