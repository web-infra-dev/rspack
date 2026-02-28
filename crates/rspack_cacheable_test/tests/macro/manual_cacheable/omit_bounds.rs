use dashmap::DashMap;
use rspack_cacheable::{from_bytes, to_bytes, with::AsMap};

// reference: https://github.com/rkyv/rkyv/blob/739f53928d7c9c870b1d2072a9b73c80466f2a87/rkyv/examples/json_like_schema.rs#L45
#[derive(
  rspack_cacheable::__private::rkyv::Archive,
  rspack_cacheable::__private::rkyv::Deserialize,
  rspack_cacheable::__private::rkyv::Serialize,
)]
#[rkyv(crate=rspack_cacheable::__private::rkyv)]
#[rkyv(serialize_bounds(
    __S: rspack_cacheable::__private::rkyv::ser::Writer + rspack_cacheable::__private::rkyv::ser::Allocator + rspack_cacheable::__private::rkyv::rancor::Fallible<Error = rspack_cacheable::Error>,
  ))]
#[rkyv(deserialize_bounds(
      __D: rspack_cacheable::__private::rkyv::rancor::Fallible<Error = rspack_cacheable::Error>
    ))]
#[rkyv(bytecheck(
    bounds(
        __C: rspack_cacheable::__private::rkyv::validation::ArchiveContext + rspack_cacheable::__private::rkyv::rancor::Fallible<Error = rspack_cacheable::Error>,
    )
  ))]
#[derive(Debug, Clone)]
struct Value {
  id: String,
  #[rkyv(omit_bounds)]
  #[rkyv(with=AsMap)]
  map: DashMap<String, Value>,
  #[rkyv(omit_bounds)]
  children: Vec<Value>,
}

#[test]
fn omit_bounds_attr() {
  let map = DashMap::default();
  map.insert(
    String::from("a"),
    Value {
      id: String::from("a"),
      map: DashMap::default(),
      children: vec![],
    },
  );
  map.insert(
    String::from("b"),
    Value {
      id: String::from("b"),
      map: DashMap::default(),
      children: vec![],
    },
  );
  let value = Value {
    id: String::from("root"),
    children: map.iter().map(|item| item.value().clone()).collect(),
    map,
  };
  let bytes = to_bytes(&value, &()).unwrap();
  let new_value: Value = from_bytes(&bytes, &()).unwrap();

  assert_eq!(value.id, new_value.id);
  for (key, value) in new_value.map {
    assert!(key == "a" || key == "b");
    assert!(value.id == "a" || value.id == "b");
    assert_eq!(value.map.len(), 0);
    assert_eq!(value.children.len(), 0);
  }
  for value in new_value.children {
    assert!(value.id == "a" || value.id == "b");
    assert_eq!(value.map.len(), 0);
    assert_eq!(value.children.len(), 0);
  }
}
