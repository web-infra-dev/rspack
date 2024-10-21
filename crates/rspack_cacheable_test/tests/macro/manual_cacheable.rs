use rspack_cacheable::{
  from_bytes, to_bytes,
  with::{AsMap, AsRefStr, AsRefStrConverter},
};

#[test]
fn basic_macro_feature() {
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

  #[allow(non_upper_case_globals)]
  const _: () = {
    use rkyv::{
      rancor::Fallible,
      with::{ArchiveWith, DeserializeWith, SerializeWith},
      Archive, Deserialize, Place, Serialize,
    };
    use rspack_cacheable::__private::rkyv;
    impl Archive for Person {
      type Archived = <AsRefStr as ArchiveWith<Person>>::Archived;
      type Resolver = <AsRefStr as ArchiveWith<Person>>::Resolver;
      #[inline]
      fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        <AsRefStr as ArchiveWith<Person>>::resolve_with(self, resolver, out)
      }
    }
    impl<S> Serialize<S> for Person
    where
      S: Fallible + ?Sized,
      AsRefStr: SerializeWith<Person, S>,
    {
      #[inline]
      fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        AsRefStr::serialize_with(self, serializer)
      }
    }
    impl<D> Deserialize<Person, D> for <AsRefStr as ArchiveWith<Person>>::Archived
    where
      D: Fallible + ?Sized,
      AsRefStr: DeserializeWith<<AsRefStr as ArchiveWith<Person>>::Archived, Person, D>,
    {
      #[inline]
      fn deserialize(&self, deserializer: &mut D) -> Result<Person, D::Error> {
        AsRefStr::deserialize_with(self, deserializer)
      }
    }
  };

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

  #[derive(Debug, PartialEq, Eq)]
  struct Person<T: Default> {
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

  #[allow(non_upper_case_globals)]
  const _: () = {
    use rkyv::{
      rancor::Fallible,
      with::{ArchiveWith, DeserializeWith, SerializeWith},
      Archive, Deserialize, Place, Serialize,
    };
    use rspack_cacheable::__private::rkyv;
    impl<T: Default> Archive for Person<T> {
      type Archived = <AsRefStr as ArchiveWith<Person<T>>>::Archived;
      type Resolver = <AsRefStr as ArchiveWith<Person<T>>>::Resolver;
      #[inline]
      fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        <AsRefStr as ArchiveWith<Person<T>>>::resolve_with(self, resolver, out)
      }
    }
    impl<T: Default, S> Serialize<S> for Person<T>
    where
      S: Fallible + ?Sized,
      AsRefStr: SerializeWith<Person<T>, S>,
    {
      #[inline]
      fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        AsRefStr::serialize_with(self, serializer)
      }
    }
    impl<T: Default, D> Deserialize<Person<T>, D> for <AsRefStr as ArchiveWith<Person<T>>>::Archived
    where
      D: Fallible + ?Sized,
      AsRefStr: DeserializeWith<<AsRefStr as ArchiveWith<Person<T>>>::Archived, Person<T>, D>,
    {
      #[inline]
      fn deserialize(&self, deserializer: &mut D) -> Result<Person<T>, D::Error> {
        AsRefStr::deserialize_with(self, deserializer)
      }
    }
  };

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

  // reference: https://github.com/rkyv/rkyv/blob/739f53928d7c9c870b1d2072a9b73c80466f2a87/rkyv/examples/json_like_schema.rs#L45
  #[derive(
    rspack_cacheable::__private::rkyv::Archive,
    rspack_cacheable::__private::rkyv::Deserialize,
    rspack_cacheable::__private::rkyv::Serialize,
  )]
  #[rkyv(crate=rspack_cacheable::__private::rkyv)]
  #[rkyv(serialize_bounds(
    __S: rspack_cacheable::__private::rkyv::ser::Writer + rspack_cacheable::__private::rkyv::ser::Allocator + rspack_cacheable::__private::rkyv::rancor::Fallible<Error = rspack_cacheable::SerializeError>,
  ))]
  #[rkyv(deserialize_bounds(
      __D: rspack_cacheable::__private::rkyv::rancor::Fallible<Error = rspack_cacheable::DeserializeError>
    ))]
  #[rkyv(bytecheck(
    bounds(
        __C: rspack_cacheable::__private::rkyv::validation::ArchiveContext + rspack_cacheable::__private::rkyv::rancor::Fallible<Error = rspack_cacheable::DeserializeError>,
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
