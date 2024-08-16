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
  #[archive(check_bytes, crate = "rspack_cacheable::__private::rkyv")]
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
    use rspack_cacheable::__private::rkyv;
    impl rkyv::Archive for Person {
      type Archived = <AsRefStr as rkyv::with::ArchiveWith<Person>>::Archived;
      type Resolver = <AsRefStr as rkyv::with::ArchiveWith<Person>>::Resolver;
      #[inline]
      unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
        <rkyv::with::With<Person, AsRefStr>>::cast(self).resolve(pos, resolver, out)
      }
    }
    impl<S> rkyv::Serialize<S> for Person
    where
      rkyv::with::With<Person, AsRefStr>: rkyv::Serialize<S>,
      S: rkyv::Fallible + ?Sized,
    {
      #[inline]
      fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        <rkyv::with::With<Person, AsRefStr>>::cast(self).serialize(serializer)
      }
    }
    impl<D: rkyv::Fallible + ?Sized> rkyv::Deserialize<Person, D>
      for <AsRefStr as rkyv::with::ArchiveWith<Person>>::Archived
    where
      rkyv::with::With<Person, AsRefStr>: rkyv::Archive,
      rkyv::Archived<rkyv::with::With<Person, AsRefStr>>:
        rkyv::Deserialize<rkyv::with::With<Person, AsRefStr>, D>,
    {
      #[inline]
      fn deserialize(&self, _deserializer: &mut D) -> Result<Person, D::Error> {
        Ok(
          rkyv::Deserialize::<rkyv::with::With<Person, AsRefStr>, D>::deserialize(
            self,
            _deserializer,
          )?
          .into_inner(),
        )
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
