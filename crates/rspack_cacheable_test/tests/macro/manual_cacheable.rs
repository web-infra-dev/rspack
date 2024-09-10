use rspack_cacheable::{
  from_bytes, to_bytes,
  with::{AsRefStr, AsRefStrConverter},
};

#[test]
fn test_manual_cacheable() {
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
fn test_manual_cacheable_with_macro() {
  #[derive(Debug, PartialEq, Eq)]
  struct UnCacheable {}

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
        uncacheable: UnCacheable {},
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
    uncacheable: UnCacheable {},
  };
  let bytes = to_bytes(&a, &()).unwrap();
  let deserialize_a = from_bytes(&bytes, &()).unwrap();
  assert_eq!(a, deserialize_a);
}
