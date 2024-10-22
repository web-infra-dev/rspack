use rspack_cacheable::{cacheable, from_bytes, to_bytes};

#[test]
fn test_manual_cacheable_dyn_macro() {
  struct Context;

  trait Animal: rspack_cacheable::r#dyn::SerializeDyn {
    fn color(&self) -> &str;
    fn name(&self) -> &str;

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
      fn serialize_unsized(&self, serializer: &mut Serializer) -> Result<usize, SerializeError> {
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
      ) -> Result<(), DeserializeError> {
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

  impl Animal for Dog {
    fn color(&self) -> &str {
      &self.color
    }
    fn name(&self) -> &str {
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
        core::mem::transmute(ptr_meta::metadata(
          core::ptr::null::<Archived<Dog>>() as *const <dyn Animal as ArchiveUnsized>::Archived
        ))
      }
    }
    inventory::submit! { DynEntry::new(*__DYN_ID_DOG_ANIMAL, get_vtable()) }
    inventory::submit! { CheckBytesEntry::new(get_vtable(), default_check_bytes_dyn::<Archived<Dog>>) }

    impl DeserializeDyn<dyn Animal> for ArchivedDog
    where
      ArchivedDog: Deserialize<Dog, Deserializer>,
    {
      fn deserialize_dyn(
        &self,
        deserializer: &mut Deserializer,
        out: *mut dyn Animal,
      ) -> Result<(), DeserializeError> {
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

  static __DYN_ID_CAT_ANIMAL: std::sync::LazyLock<u64> = std::sync::LazyLock::new(|| {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    module_path!().hash(&mut hasher);
    line!().hash(&mut hasher);
    hasher.finish()
  });

  impl Animal for Cat {
    fn color(&self) -> &str {
      &self.color
    }
    fn name(&self) -> &str {
      "cat"
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
        core::mem::transmute(ptr_meta::metadata(
          core::ptr::null::<Archived<Cat>>() as *const <dyn Animal as ArchiveUnsized>::Archived
        ))
      }
    }
    inventory::submit! { DynEntry::new(*__DYN_ID_CAT_ANIMAL, get_vtable()) }
    inventory::submit! { CheckBytesEntry::new(get_vtable(), default_check_bytes_dyn::<Archived<Cat>>) }

    impl DeserializeDyn<dyn Animal> for ArchivedCat
    where
      ArchivedCat: Deserialize<Cat, Deserializer>,
    {
      fn deserialize_dyn(
        &self,
        deserializer: &mut Deserializer,
        out: *mut dyn Animal,
      ) -> Result<(), DeserializeError> {
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
