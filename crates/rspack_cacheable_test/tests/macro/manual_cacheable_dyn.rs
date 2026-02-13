use rspack_cacheable::{
  CacheableContext, r#dyn::VTablePtr, enable_cacheable as cacheable, from_bytes, to_bytes,
};

#[test]
#[cfg_attr(miri, ignore)]
fn test_manual_cacheable_dyn_macro() {
  struct Context;
  impl CacheableContext for Context {
    fn project_root(&self) -> Option<&std::path::Path> {
      None
    }
  }

  trait Animal: rspack_cacheable::r#dyn::SerializeDyn {
    fn color(&self) -> &str;
    fn name(&self) -> &str;

    #[doc(hidden)]
    fn __dyn_id(&self) -> u64;
  }

  const _: () = {
    use std::alloc::{Layout, LayoutError};

    use rspack_cacheable::{
      __private::rkyv::{
        ArchiveUnsized, ArchivedMetadata, DeserializeUnsized, Portable, SerializeUnsized,
        bytecheck::CheckBytes,
        ptr_meta,
        traits::{ArchivePointee, LayoutRaw},
      },
      Deserializer, Error, Serializer, Validator,
      r#dyn::{ArchivedDynMetadata, DeserializeDyn, validation::CHECK_BYTES_REGISTRY},
    };

    unsafe impl ptr_meta::Pointee for dyn Animal {
      type Metadata = ptr_meta::DynMetadata<Self>;
    }

    impl ArchiveUnsized for dyn Animal {
      type Archived = dyn DeserializeAnimal;

      fn archived_metadata(&self) -> ArchivedMetadata<Self> {
        ArchivedDynMetadata::new(Animal::__dyn_id(self))
      }
    }

    impl LayoutRaw for dyn Animal {
      fn layout_raw(
        metadata: <Self as ptr_meta::Pointee>::Metadata,
      ) -> Result<Layout, LayoutError> {
        Ok(metadata.layout())
      }
    }

    impl SerializeUnsized<Serializer<'_>> for dyn Animal {
      fn serialize_unsized(&self, serializer: &mut Serializer) -> Result<usize, Error> {
        self.serialize_dyn(serializer)
      }
    }

    pub trait DeserializeAnimal: DeserializeDyn<dyn Animal> + Portable {}
    unsafe impl ptr_meta::Pointee for dyn DeserializeAnimal {
      type Metadata = ptr_meta::DynMetadata<Self>;
    }

    impl<T: DeserializeDyn<dyn Animal> + Portable> DeserializeAnimal for T {}

    impl ArchivePointee for dyn DeserializeAnimal {
      type ArchivedMetadata = ArchivedDynMetadata<Self>;

      fn pointer_metadata(
        archived: &Self::ArchivedMetadata,
      ) -> <Self as ptr_meta::Pointee>::Metadata {
        archived.lookup_metadata()
      }
    }

    impl DeserializeUnsized<dyn Animal, Deserializer> for dyn DeserializeAnimal {
      unsafe fn deserialize_unsized(
        &self,
        deserializer: &mut Deserializer,
        out: *mut dyn Animal,
      ) -> Result<(), Error> {
        self.deserialize_dyn(deserializer, out)
      }

      fn deserialize_metadata(&self) -> <dyn Animal as ptr_meta::Pointee>::Metadata {
        self.deserialized_pointer_metadata()
      }
    }

    impl LayoutRaw for dyn DeserializeAnimal {
      fn layout_raw(
        metadata: <Self as ptr_meta::Pointee>::Metadata,
      ) -> Result<Layout, LayoutError> {
        Ok(metadata.layout())
      }
    }

    // CheckBytes
    unsafe impl CheckBytes<Validator<'_>> for dyn DeserializeAnimal {
      #[inline]
      unsafe fn check_bytes(value: *const Self, context: &mut Validator) -> Result<(), Error> {
        let vtable = VTablePtr::new(ptr_meta::metadata(value));
        if let Some(check_bytes_dyn) = CHECK_BYTES_REGISTRY.get(&vtable) {
          unsafe { check_bytes_dyn(value.cast(), context)? };
          Ok(())
        } else {
          Err(Error::DynCheckBytesNotRegister)
        }
      }
    }
  };

  #[cacheable]
  struct Dog {
    color: String,
  }

  const __DYN_ID_DOG_ANIMAL: u64 =
    xxhash_rust::const_xxh64::xxh64(concat!(module_path!(), ":", line!()).as_bytes(), 0);

  impl Animal for Dog {
    fn color(&self) -> &str {
      &self.color
    }
    fn name(&self) -> &str {
      "dog"
    }

    fn __dyn_id(&self) -> u64 {
      __DYN_ID_DOG_ANIMAL
    }
  }

  const _: () = {
    use rspack_cacheable::{
      __private::{
        inventory,
        rkyv::{ArchiveUnsized, Archived, Deserialize, DeserializeUnsized, ptr_meta},
      },
      Deserializer, Error,
      r#dyn::{
        DeserializeDyn, DynEntry,
        validation::{CheckBytesEntry, default_check_bytes_dyn},
      },
    };

    const fn get_vtable() -> VTablePtr {
      VTablePtr::new(ptr_meta::metadata(
        core::ptr::null::<Archived<Dog>>() as *const <dyn Animal as ArchiveUnsized>::Archived
      ))
    }
    inventory::submit! { DynEntry::new(__DYN_ID_DOG_ANIMAL, get_vtable()) }
    inventory::submit! { CheckBytesEntry::new(get_vtable(), default_check_bytes_dyn::<Archived<Dog>>) }

    impl DeserializeDyn<dyn Animal> for ArchivedDog
    where
      ArchivedDog: Deserialize<Dog, Deserializer>,
    {
      fn deserialize_dyn(
        &self,
        deserializer: &mut Deserializer,
        out: *mut dyn Animal,
      ) -> Result<(), Error> {
        unsafe {
          <Self as DeserializeUnsized<Dog, _>>::deserialize_unsized(self, deserializer, out.cast())
        }
      }

      fn deserialized_pointer_metadata(&self) -> ptr_meta::DynMetadata<dyn Animal> {
        ptr_meta::metadata(core::ptr::null::<Dog>() as *const dyn Animal)
      }
    }
  };

  #[cacheable]
  struct Cat {
    color: String,
  }

  const __DYN_ID_CAT_ANIMAL: u64 =
    xxhash_rust::const_xxh64::xxh64(concat!(module_path!(), ":", line!()).as_bytes(), 0);

  impl Animal for Cat {
    fn color(&self) -> &str {
      &self.color
    }
    fn name(&self) -> &str {
      "cat"
    }

    fn __dyn_id(&self) -> u64 {
      __DYN_ID_CAT_ANIMAL
    }
  }

  const _: () = {
    use rspack_cacheable::{
      __private::{
        inventory,
        rkyv::{ArchiveUnsized, Archived, Deserialize, DeserializeUnsized, ptr_meta},
      },
      Deserializer, Error,
      r#dyn::{
        DeserializeDyn, DynEntry,
        validation::{CheckBytesEntry, default_check_bytes_dyn},
      },
    };

    const fn get_vtable() -> VTablePtr {
      VTablePtr::new(ptr_meta::metadata(
        core::ptr::null::<Archived<Cat>>() as *const <dyn Animal as ArchiveUnsized>::Archived
      ))
    }
    inventory::submit! { DynEntry::new(__DYN_ID_CAT_ANIMAL, get_vtable()) }
    inventory::submit! { CheckBytesEntry::new(get_vtable(), default_check_bytes_dyn::<Archived<Cat>>) }

    impl DeserializeDyn<dyn Animal> for ArchivedCat
    where
      ArchivedCat: Deserialize<Cat, Deserializer>,
    {
      fn deserialize_dyn(
        &self,
        deserializer: &mut Deserializer,
        out: *mut dyn Animal,
      ) -> Result<(), Error> {
        unsafe {
          <Self as DeserializeUnsized<Cat, _>>::deserialize_unsized(self, deserializer, out.cast())
        }
      }

      fn deserialized_pointer_metadata(&self) -> ptr_meta::DynMetadata<dyn Animal> {
        ptr_meta::metadata(core::ptr::null::<Cat>() as *const dyn Animal)
      }
    }
  };

  #[cacheable]
  struct Data {
    #[cacheable(with=::rspack_cacheable::with::AsCacheable)]
    animal: Box<dyn Animal>,
  }

  let dog_data = Data {
    animal: Box::new(Dog {
      color: String::from("black"),
    }),
  };
  assert_eq!(dog_data.animal.name(), "dog");
  assert_eq!(dog_data.animal.color(), "black");
  let ctx = Context {};
  let bytes = to_bytes(&dog_data, &ctx).unwrap();
  let deserialize_data = from_bytes::<Data, Context>(&bytes, &ctx).unwrap();
  assert_eq!(deserialize_data.animal.name(), "dog");
  assert_eq!(deserialize_data.animal.color(), "black");

  let cat_data = Data {
    animal: Box::new(Cat {
      color: String::from("white"),
    }),
  };
  assert_eq!(cat_data.animal.name(), "cat");
  assert_eq!(cat_data.animal.color(), "white");
  let ctx = Context {};
  let bytes = to_bytes(&cat_data, &ctx).unwrap();
  let deserialize_data = from_bytes::<Data, Context>(&bytes, &ctx).unwrap();
  assert_eq!(deserialize_data.animal.name(), "cat");
  assert_eq!(deserialize_data.animal.color(), "white");
}
