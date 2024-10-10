use std::sync::Arc;

use rkyv::{
  vec::{ArchivedVec, VecResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Place,
};
use rspack_sources::{
  OriginalSource, RawSource, Source, SourceMap, SourceMapSource, SourceMapSourceOptions,
};

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
  fn resolve_with(_field: &Arc<dyn Source>, resolver: Self::Resolver, out: Place<Self::Archived>) {
    ArchivedVec::resolve_from_len(resolver.len, resolver.inner, out)
  }
}

// TODO add cacheable to rspack-sources
impl<'a> SerializeWith<Arc<dyn Source>, CacheableSerializer<'a>> for AsPreset {
  fn serialize_with(
    field: &Arc<dyn Source>,
    serializer: &mut CacheableSerializer,
  ) -> Result<Self::Resolver, SerializeError> {
    let inner = field.as_ref().as_any();
    let bytes = if let Some(raw_source) = inner.downcast_ref::<RawSource>() {
      let data = TypeWrapperRef {
        type_name: "RawSource",
        bytes: &raw_source.buffer(),
      };
      crate::to_bytes(&data, &())?
    } else if let Some(original_source) = inner.downcast_ref::<OriginalSource>() {
      let source = original_source.source();
      let data = Some(TypeWrapperRef {
        type_name: "OriginalSource",
        bytes: source.as_bytes(),
      });
      crate::to_bytes(&data, &())?
    } else if let Some(source_map_source) = inner.downcast_ref::<SourceMapSource>() {
      let source = source_map_source.source();
      let data = Some(TypeWrapperRef {
        type_name: "SourceMapSource",
        bytes: source.as_bytes(),
      });
      crate::to_bytes(&data, &())?
    } else {
      return Err(SerializeError::SerializeFailed("unsupport rspack source"));
    };
    Ok(SourceResolver {
      inner: ArchivedVec::serialize_from_slice(&bytes, serializer)?,
      len: bytes.len(),
    })
  }
}

impl DeserializeWith<ArchivedVec<u8>, Arc<dyn Source>, CacheableDeserializer> for AsPreset {
  fn deserialize_with(
    field: &ArchivedVec<u8>,
    _de: &mut CacheableDeserializer,
  ) -> Result<Arc<dyn Source>, DeserializeError> {
    let TypeWrapper { type_name, bytes } = crate::from_bytes(field, &())?;
    match type_name.as_str() {
      // TODO change to enum
      "RawSource" => Ok(Arc::new(RawSource::from(bytes))),
      // TODO save original source name
      "OriginalSource" => Ok(Arc::new(OriginalSource::new(
        "a",
        String::from_utf8(bytes).expect("unexpect bytes"),
      ))),
      "SourceMapSource" => Ok(Arc::new(SourceMapSource::new(SourceMapSourceOptions {
        value: String::from_utf8(bytes).expect("unexpect bytes"),
        name: String::from("a"),
        source_map: SourceMap::default(),
        original_source: None,
        inner_source_map: None,
        remove_original_source: true,
      }))),
      _ => Err(DeserializeError::DeserializeFailed(
        "unsupported box source",
      )),
    }
  }
}
