use rspack_cacheable::{
  cacheable, from_bytes, to_bytes,
  with::{AsMap, AsRefStr, AsRefStrConverter},
};

#[test]
fn basic_macro_feature() {
  #[cacheable]
  #[derive(Debug, PartialEq, Eq)]
  struct Person {
    name: String,
  }

  let a = Person {
    name: String::from("a"),
  };
  let bytes = to_bytes(&a, &()).unwrap();
  let deserialize_a = from_bytes(&bytes, &()).unwrap();
  assert_eq!(a, deserialize_a);
}

#[test]
fn hashable_attr() {
  use rustc_hash::FxHashSet as HashSet;
  #[cacheable(hashable)]
  #[derive(Debug, Hash, PartialEq, Eq)]
  struct Person {
    name: String,
  }

  let mut a = HashSet::default();
  a.insert(Person {
    name: String::from("a"),
  });
  let bytes = to_bytes(&a, &()).unwrap();
  let deserialize_a: HashSet<Person> = from_bytes(&bytes, &()).unwrap();
  assert_eq!(a, deserialize_a);
}

#[test]
fn with_attr() {
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

  let a = Person {
    name: String::from("a"),
    uncacheable: UnCacheable,
  };
  let bytes = to_bytes(&a, &()).unwrap();
  let deserialize_a = from_bytes(&bytes, &()).unwrap();
  assert_eq!(a, deserialize_a);
}

#[test]
fn with_attr_with_generics() {
  #[derive(Debug, Default, PartialEq, Eq)]
  struct UnCacheable;

  #[cacheable(with=AsRefStr)]
  #[derive(Debug, PartialEq, Eq)]
  struct Person<T>
  where
    T: Default,
  {
    name: String,
    uncacheable: T,
  }
  impl<T: Default> AsRefStrConverter for Person<T> {
    fn as_str(&self) -> &str {
      &self.name
    }
    fn from_str(s: &str) -> Self {
      Self {
        name: String::from(s),
        uncacheable: Default::default(),
      }
    }
  }

  let a = Person {
    name: String::from("a"),
    uncacheable: UnCacheable,
  };
  let bytes = to_bytes(&a, &()).unwrap();
  let deserialize_a = from_bytes(&bytes, &()).unwrap();
  assert_eq!(a, deserialize_a);
}

#[test]
fn omit_bounds_attr() {
  use dashmap::DashMap;

  #[cacheable]
  #[derive(Debug, Clone)]
  struct Value {
    id: String,
    #[cacheable(omit_bounds, with=AsMap)]
    map: DashMap<String, Value>,
    #[cacheable(omit_bounds)]
    children: Vec<Value>,
  }

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
