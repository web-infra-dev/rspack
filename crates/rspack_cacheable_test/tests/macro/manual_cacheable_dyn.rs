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
    use core::alloc::Layout;
    use std::alloc::LayoutError;

    use rspack_cacheable::__private::{
      ptr_meta,
      rkyv::{
        validation::{validators::DefaultValidator, LayoutRaw},
        ArchivePointee, ArchiveUnsized, ArchivedMetadata, CheckBytes, DeserializeUnsized,
        SerializeUnsized,
      },
    };
    use rspack_cacheable::{
      r#dyn::{validation::CHECK_BYTES_REGISTRY, ArchivedDynMetadata, DeserializeDyn},
      CacheableDeserializer, CacheableSerializer, DeserializeError, SerializeError,
    };

    impl ptr_meta::Pointee for dyn Animal {
      type Metadata = ptr_meta::DynMetadata<Self>;
    }

    pub trait DeserializeAnimal: DeserializeDyn<dyn Animal> {}
    impl ptr_meta::Pointee for dyn DeserializeAnimal {
      type Metadata = ptr_meta::DynMetadata<Self>;
    }

    impl<T: DeserializeDyn<dyn Animal>> DeserializeAnimal for T {}

    impl ArchiveUnsized for dyn Animal {
      type Archived = dyn DeserializeAnimal;
      type MetadataResolver = ();

      unsafe fn resolve_metadata(
        &self,
        _: usize,
        _: Self::MetadataResolver,
        out: *mut ArchivedMetadata<Self>,
      ) {
        ArchivedDynMetadata::emplace(Animal::__dyn_id(self), out);
      }
    }

    impl SerializeUnsized<CacheableSerializer> for dyn Animal {
      fn serialize_unsized(
        &self,
        mut serializer: &mut CacheableSerializer,
      ) -> Result<usize, SerializeError> {
        self.serialize_dyn(&mut serializer)
      }

      fn serialize_metadata(
        &self,
        _: &mut CacheableSerializer,
      ) -> Result<Self::MetadataResolver, SerializeError> {
        Ok(())
      }
    }

    impl ArchivePointee for dyn DeserializeAnimal {
      type ArchivedMetadata = ArchivedDynMetadata<Self>;

      fn pointer_metadata(
        archived: &Self::ArchivedMetadata,
      ) -> <Self as ptr_meta::Pointee>::Metadata {
        archived.pointer_metadata()
      }
    }

    impl DeserializeUnsized<dyn Animal, CacheableDeserializer> for dyn DeserializeAnimal {
      unsafe fn deserialize_unsized(
        &self,
        mut deserializer: &mut CacheableDeserializer,
        mut alloc: impl FnMut(Layout) -> *mut u8,
      ) -> Result<*mut (), DeserializeError> {
        self.deserialize_dyn(&mut deserializer, &mut alloc)
      }

      fn deserialize_metadata(
        &self,
        mut deserializer: &mut CacheableDeserializer,
      ) -> Result<<dyn Animal as ptr_meta::Pointee>::Metadata, DeserializeError> {
        self.deserialize_dyn_metadata(&mut deserializer)
      }
    }

    // CheckBytes
    impl LayoutRaw for dyn DeserializeAnimal {
      fn layout_raw(
        metadata: <Self as ptr_meta::Pointee>::Metadata,
      ) -> Result<Layout, LayoutError> {
        Ok(metadata.layout())
      }
    }
    impl CheckBytes<DefaultValidator<'_>> for dyn DeserializeAnimal {
      type Error = DeserializeError;
      #[inline]
      unsafe fn check_bytes<'a>(
        value: *const Self,
        context: &mut DefaultValidator<'_>,
      ) -> Result<&'a Self, Self::Error> {
        let vtable: usize = core::mem::transmute(ptr_meta::metadata(value));
        if let Some(check_bytes_dyn) = CHECK_BYTES_REGISTRY.get(&vtable) {
          check_bytes_dyn(value.cast(), context)?;
          Ok(&*value)
        } else {
          Err(DeserializeError::CheckBytesError)
        }
      }
    }
  };

  #[cacheable]
  struct Dog {
    color: String,
  }

  const __DYN_ID_DOG_ANIMAL: std::sync::LazyLock<u64> = std::sync::LazyLock::new(|| {
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
    use core::alloc::Layout;

    use rspack_cacheable::__private::{
      inventory, ptr_meta,
      rkyv::{ArchiveUnsized, Archived, Deserialize},
    };
    use rspack_cacheable::{
      r#dyn::{
        validation::{default_check_bytes_dyn, CheckBytesEntry},
        DeserializeDyn, DynEntry,
      },
      CacheableDeserializer, DeserializeError,
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
      ArchivedDog: Deserialize<Dog, CacheableDeserializer>,
    {
      unsafe fn deserialize_dyn(
        &self,
        deserializer: &mut CacheableDeserializer,
        alloc: &mut dyn FnMut(Layout) -> *mut u8,
      ) -> Result<*mut (), DeserializeError> {
        let result = alloc(Layout::new::<Dog>()).cast::<Dog>();
        assert!(!result.is_null());
        result.write(self.deserialize(deserializer)?);
        Ok(result as *mut ())
      }

      fn deserialize_dyn_metadata(
        &self,
        _: &mut CacheableDeserializer,
      ) -> Result<<dyn Animal as ptr_meta::Pointee>::Metadata, DeserializeError> {
        unsafe {
          Ok(core::mem::transmute(ptr_meta::metadata(
            core::ptr::null::<Dog>() as *const dyn Animal,
          )))
        }
      }
    }
  };

  #[cacheable]
  struct Cat {
    color: String,
  }

  const __DYN_ID_CAT_ANIMAL: std::sync::LazyLock<u64> = std::sync::LazyLock::new(|| {
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
    use core::alloc::Layout;

    use rspack_cacheable::__private::{
      inventory, ptr_meta,
      rkyv::{ArchiveUnsized, Archived, Deserialize},
    };
    use rspack_cacheable::{
      r#dyn::{
        validation::{default_check_bytes_dyn, CheckBytesEntry},
        DeserializeDyn, DynEntry,
      },
      CacheableDeserializer, DeserializeError,
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
      ArchivedCat: Deserialize<Cat, CacheableDeserializer>,
    {
      unsafe fn deserialize_dyn(
        &self,
        deserializer: &mut CacheableDeserializer,
        alloc: &mut dyn FnMut(Layout) -> *mut u8,
      ) -> Result<*mut (), DeserializeError> {
        let result = alloc(Layout::new::<Cat>()).cast::<Cat>();
        assert!(!result.is_null());
        result.write(self.deserialize(deserializer)?);
        Ok(result as *mut ())
      }

      fn deserialize_dyn_metadata(
        &self,
        _: &mut CacheableDeserializer,
      ) -> Result<<dyn Animal as ptr_meta::Pointee>::Metadata, DeserializeError> {
        unsafe {
          Ok(core::mem::transmute(ptr_meta::metadata(
            core::ptr::null::<Cat>() as *const dyn Animal,
          )))
        }
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
