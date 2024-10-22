use rspack_cacheable::{cacheable, from_bytes, to_bytes};

#[test]
fn test_manual_cacheable_dyn_macro_with_generics() {
  struct Context;

  trait Animal<T = ()>: rspack_cacheable::r#dyn::SerializeDyn {
    fn color(&self) -> &str;
    fn name(&self) -> T;

    #[doc(hidden)]
    fn __dyn_id(&self) -> u64;
  }

  const _: () = {
    use std::alloc::{Layout, LayoutError};

    use rspack_cacheable::__private::rkyv::{
      bytecheck::CheckBytes,
      ptr_meta,
      traits::{ArchivePointee, LayoutRaw},
      ArchiveUnsized, ArchivedMetadata, DeserializeUnsized, Portable, SerializeUnsized,
    };
    use rspack_cacheable::{
      r#dyn::{validation::CHECK_BYTES_REGISTRY, ArchivedDynMetadata, DeserializeDyn},
      DeserializeError, Deserializer, SerializeError, Serializer, Validator,
    };

    unsafe impl<T> ptr_meta::Pointee for dyn Animal<T> {
      type Metadata = ptr_meta::DynMetadata<Self>;
    }

    impl<T> ArchiveUnsized for dyn Animal<T> {
      type Archived = dyn DeserializeAnimal<T>;

      fn archived_metadata(&self) -> ArchivedMetadata<Self> {
        ArchivedDynMetadata::new(Animal::__dyn_id(self))
      }
    }

    impl<T> LayoutRaw for dyn Animal<T> {
      fn layout_raw(
        metadata: <Self as ptr_meta::Pointee>::Metadata,
      ) -> Result<Layout, LayoutError> {
        Ok(metadata.layout())
      }
    }

    impl<T> SerializeUnsized<Serializer<'_>> for dyn Animal<T> {
      fn serialize_unsized(&self, serializer: &mut Serializer) -> Result<usize, SerializeError> {
        self.serialize_dyn(serializer)
      }
    }

    pub trait DeserializeAnimal<T>: DeserializeDyn<dyn Animal<T>> + Portable {}
    unsafe impl<T> ptr_meta::Pointee for dyn DeserializeAnimal<T> {
      type Metadata = ptr_meta::DynMetadata<Self>;
    }

    impl<T, O: DeserializeDyn<dyn Animal<T>> + Portable> DeserializeAnimal<T> for O {}

    impl<T> ArchivePointee for dyn DeserializeAnimal<T> {
      type ArchivedMetadata = ArchivedDynMetadata<Self>;

      fn pointer_metadata(
        archived: &Self::ArchivedMetadata,
      ) -> <Self as ptr_meta::Pointee>::Metadata {
        archived.lookup_metadata()
      }
    }

    impl<T> DeserializeUnsized<dyn Animal<T>, Deserializer> for dyn DeserializeAnimal<T> {
      unsafe fn deserialize_unsized(
        &self,
        deserializer: &mut Deserializer,
        out: *mut dyn Animal<T>,
      ) -> Result<(), DeserializeError> {
        self.deserialize_dyn(deserializer, out)
      }

      fn deserialize_metadata(&self) -> <dyn Animal<T> as ptr_meta::Pointee>::Metadata {
        self.deserialized_pointer_metadata()
      }
    }

    impl<T> LayoutRaw for dyn DeserializeAnimal<T> {
      fn layout_raw(
        metadata: <Self as ptr_meta::Pointee>::Metadata,
      ) -> Result<Layout, LayoutError> {
        Ok(metadata.layout())
      }
    }

    // CheckBytes
    unsafe impl<T> CheckBytes<Validator<'_>> for dyn DeserializeAnimal<T> {
      #[inline]
      unsafe fn check_bytes(
        value: *const Self,
        context: &mut Validator,
      ) -> Result<(), DeserializeError> {
        let vtable: usize = std::mem::transmute(ptr_meta::metadata(value));
        if let Some(check_bytes_dyn) = CHECK_BYTES_REGISTRY.get(&vtable) {
          check_bytes_dyn(value.cast(), context)?;
          Ok(())
        } else {
          Err(DeserializeError::DynCheckBytesNotRegister)
        }
      }
    }
  };

  #[cacheable]
  struct Dog {
    color: String,
  }

  static __DYN_ID_DOG_ANIMAL: std::sync::LazyLock<u64> = std::sync::LazyLock::new(|| {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    module_path!().hash(&mut hasher);
    line!().hash(&mut hasher);
    hasher.finish()
  });

  impl Animal<&'static str> for Dog {
    fn color(&self) -> &str {
      &self.color
    }
    fn name(&self) -> &'static str {
      "dog"
    }
    fn __dyn_id(&self) -> u64 {
      *__DYN_ID_DOG_ANIMAL
    }
  }

  const _: () = {
    use rspack_cacheable::__private::{
      inventory,
      rkyv::{ptr_meta, ArchiveUnsized, Archived, Deserialize, DeserializeUnsized},
    };
    use rspack_cacheable::{
      r#dyn::{
        validation::{default_check_bytes_dyn, CheckBytesEntry},
        DeserializeDyn, DynEntry,
      },
      DeserializeError, Deserializer,
    };

    fn get_vtable() -> usize {
      unsafe {
        core::mem::transmute(ptr_meta::metadata(core::ptr::null::<Archived<Dog>>()
          as *const <dyn Animal<&'static str> as ArchiveUnsized>::Archived))
      }
    }
    inventory::submit! { DynEntry::new(*__DYN_ID_DOG_ANIMAL, get_vtable()) }
    inventory::submit! { CheckBytesEntry::new(get_vtable(), default_check_bytes_dyn::<Archived<Dog>>) }

    impl DeserializeDyn<dyn Animal<&'static str>> for ArchivedDog
    where
      ArchivedDog: Deserialize<Dog, Deserializer>,
    {
      fn deserialize_dyn(
        &self,
        deserializer: &mut Deserializer,
        out: *mut dyn Animal<&'static str>,
      ) -> Result<(), DeserializeError> {
        unsafe {
          <Self as DeserializeUnsized<Dog, _>>::deserialize_unsized(self, deserializer, out.cast())
        }
      }

      fn deserialized_pointer_metadata(&self) -> ptr_meta::DynMetadata<dyn Animal<&'static str>> {
        ptr_meta::metadata(core::ptr::null::<Dog>() as *const dyn Animal<&'static str>)
      }
    }
  };

  #[cacheable]
  struct Cat {
    color: String,
  }

  static __DYN_ID_CAT_ANIMAL: std::sync::LazyLock<u64> = std::sync::LazyLock::new(|| {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    module_path!().hash(&mut hasher);
    line!().hash(&mut hasher);
    hasher.finish()
  });

  impl Animal<String> for Cat {
    fn color(&self) -> &str {
      &self.color
    }
    fn name(&self) -> String {
      String::from("cat")
    }
    fn __dyn_id(&self) -> u64 {
      *__DYN_ID_CAT_ANIMAL
    }
  }

  const _: () = {
    use rspack_cacheable::__private::{
      inventory,
      rkyv::{ptr_meta, ArchiveUnsized, Archived, Deserialize, DeserializeUnsized},
    };
    use rspack_cacheable::{
      r#dyn::{
        validation::{default_check_bytes_dyn, CheckBytesEntry},
        DeserializeDyn, DynEntry,
      },
      DeserializeError, Deserializer,
    };

    fn get_vtable() -> usize {
      unsafe {
        core::mem::transmute(ptr_meta::metadata(core::ptr::null::<Archived<Cat>>()
          as *const <dyn Animal<String> as ArchiveUnsized>::Archived))
      }
    }
    inventory::submit! { DynEntry::new(*__DYN_ID_CAT_ANIMAL, get_vtable()) }
    inventory::submit! { CheckBytesEntry::new(get_vtable(), default_check_bytes_dyn::<Archived<Cat>>)}

    impl DeserializeDyn<dyn Animal<String>> for ArchivedCat
    where
      ArchivedCat: Deserialize<Cat, Deserializer>,
    {
      fn deserialize_dyn(
        &self,
        deserializer: &mut Deserializer,
        out: *mut dyn Animal<String>,
      ) -> Result<(), DeserializeError> {
        unsafe {
          <Self as DeserializeUnsized<Cat, _>>::deserialize_unsized(self, deserializer, out.cast())
        }
      }

      fn deserialized_pointer_metadata(&self) -> ptr_meta::DynMetadata<dyn Animal<String>> {
        ptr_meta::metadata(core::ptr::null::<Cat>() as *const dyn Animal<String>)
      }
    }
  };

  #[cacheable]
  struct Data {
    animal_1: Box<dyn Animal<&'static str>>,
    animal_2: Box<dyn Animal<String>>,
  }

  let data = Data {
    animal_1: Box::new(Dog {
      color: String::from("black"),
    }),
    animal_2: Box::new(Cat {
      color: String::from("white"),
    }),
  };
  assert_eq!(data.animal_1.name(), "dog");
  assert_eq!(data.animal_1.color(), "black");
  assert_eq!(data.animal_2.name(), "cat");
  assert_eq!(data.animal_2.color(), "white");
  let ctx = Context {};
  let bytes = to_bytes(&data, &ctx).unwrap();
  let deserialize_data = from_bytes::<Data, Context>(&bytes, &ctx).unwrap();
  assert_eq!(deserialize_data.animal_1.name(), "dog");
  assert_eq!(deserialize_data.animal_1.color(), "black");
  assert_eq!(deserialize_data.animal_2.name(), "cat");
  assert_eq!(deserialize_data.animal_2.color(), "white");
}
