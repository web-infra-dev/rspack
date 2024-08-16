use std::sync::Arc;

use rkyv::{
  vec::{ArchivedVec, VecResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};
use rspack_sources::{OriginalSource, RawSource, Source};

use super::AsPreset;
use crate::{
  utils::{TypeWrapper, TypeWrapperRef},
  CacheableDeserializer, CacheableSerializer, DeserializeError, SerializeError,
};

pub struct SourceResolver {
  inner: VecResolver,
  len: usize,
}

/*const _: () = {
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
      ArchivedDynMetadata::emplace(Animal::__type_id(self), out);
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
    fn layout_raw(metadata: <Self as ptr_meta::Pointee>::Metadata) -> Result<Layout, LayoutError> {
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
};*/

impl ArchiveWith<Arc<dyn Source>> for AsPreset {
  type Archived = ArchivedVec<u8>;
  type Resolver = SourceResolver;

  #[inline]
  unsafe fn resolve_with(
    _field: &Arc<dyn Source>,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    ArchivedVec::resolve_from_len(resolver.len, pos, resolver.inner, out)
  }
}

impl SerializeWith<Arc<dyn Source>, CacheableSerializer> for AsPreset {
  #[inline]
  fn serialize_with(
    field: &Arc<dyn Source>,
    serializer: &mut CacheableSerializer,
  ) -> Result<Self::Resolver, SerializeError> {
    let inner = field.as_ref().as_any();
    let context = unsafe { serializer.context::<()>() };
    let mut bytes = vec![];
    if let Some(raw_source) = inner.downcast_ref::<RawSource>() {
      let data = match raw_source {
        RawSource::Buffer(buf) => {
          // TODO try avoid clone
          TypeWrapperRef {
            type_name: "RawSource::Buffer",
            bytes: buf,
          }
        }
        RawSource::Source(source) => TypeWrapperRef {
          type_name: "RawSource::Source",
          bytes: source.as_bytes(),
        },
      };
      bytes = crate::to_bytes(&data, context)?;
    }
    if let Some(original_source) = inner.downcast_ref::<OriginalSource>() {
      let source = original_source.source();
      let data = Some(TypeWrapperRef {
        type_name: "OriginalSource",
        bytes: source.as_bytes(),
      });
      bytes = crate::to_bytes(&data, context)?;
    }
    Ok(SourceResolver {
      inner: ArchivedVec::serialize_from_slice(&bytes, serializer)?,
      len: bytes.len(),
    })
  }
}

impl DeserializeWith<ArchivedVec<u8>, Arc<dyn Source>, CacheableDeserializer> for AsPreset {
  #[inline]
  fn deserialize_with(
    field: &ArchivedVec<u8>,
    de: &mut CacheableDeserializer,
  ) -> Result<Arc<dyn Source>, DeserializeError> {
    let context = unsafe { de.context::<()>() };
    let TypeWrapper { type_name, bytes } = crate::from_bytes(field, context)?;
    match type_name.as_str() {
      // TODO change to enum
      "RawSource::Buffer" => Ok(Arc::new(RawSource::Buffer(bytes))),
      "RawSource::Source" => Ok(Arc::new(RawSource::Source(
        String::from_utf8(bytes).expect("unexpect bytes"),
      ))),
      // TODO save original source name
      "OriginalSource" => Ok(Arc::new(OriginalSource::new(
        "a",
        String::from_utf8(bytes).expect("unexpect bytes"),
      ))),
      _ => Err(DeserializeError::DeserializeFailed(
        "unsupported box source",
      )),
    }
  }
}
