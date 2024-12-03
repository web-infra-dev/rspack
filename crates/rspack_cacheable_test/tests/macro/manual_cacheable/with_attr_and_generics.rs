use rspack_cacheable::{
  from_bytes, to_bytes,
  with::{AsRefStr, AsRefStrConverter},
};

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

#[test]
fn with_attr_with_generics() {
  let a = Person {
    name: String::from("a"),
    uncacheable: UnCacheable,
  };
  let bytes = to_bytes(&a, &()).unwrap();
  let deserialize_a = from_bytes(&bytes, &()).unwrap();
  assert_eq!(a, deserialize_a);
}
