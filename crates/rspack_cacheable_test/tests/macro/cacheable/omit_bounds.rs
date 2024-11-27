use dashmap::DashMap;
use rspack_cacheable::{cacheable, from_bytes, to_bytes, with::AsMap};

#[cacheable]
#[derive(Debug, Clone)]
struct Value {
  id: String,
  #[cacheable(omit_bounds, with=AsMap)]
  map: DashMap<String, Value>,
  #[cacheable(omit_bounds)]
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
