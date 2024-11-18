use std::sync::Arc;

use rkyv::{
  rancor::Fallible,
  ser::{Allocator, Writer},
  vec::{ArchivedVec, VecResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Place,
};
use rspack_sources::{
  OriginalSource, RawSource, Source, SourceMap, SourceMapSource, WithoutOriginalOptions,
};

use super::AsPreset;
use crate::{
  utils::{TypeWrapper, TypeWrapperRef},
  DeserializeError, SerializeError,
};

pub struct SourceResolver {
  inner: VecResolver,
  len: usize,
}

impl ArchiveWith<Arc<dyn Source>> for AsPreset {
  type Archived = ArchivedVec<u8>;
  type Resolver = SourceResolver;

  #[inline]
  fn resolve_with(_field: &Arc<dyn Source>, resolver: Self::Resolver, out: Place<Self::Archived>) {
    ArchivedVec::resolve_from_len(resolver.len, resolver.inner, out)
  }
}

// TODO add cacheable to rspack-sources
impl<S> SerializeWith<Arc<dyn Source>, S> for AsPreset
where
  S: Fallible<Error = SerializeError> + Writer + Allocator,
{
  fn serialize_with(
    field: &Arc<dyn Source>,
    serializer: &mut S,
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
      return Err(SerializeError::MessageError("unsupported rspack source"));
    };
    Ok(SourceResolver {
      inner: ArchivedVec::serialize_from_slice(&bytes, serializer)?,
      len: bytes.len(),
    })
  }
}

impl<D> DeserializeWith<ArchivedVec<u8>, Arc<dyn Source>, D> for AsPreset
where
  D: Fallible<Error = DeserializeError>,
{
  fn deserialize_with(
    field: &ArchivedVec<u8>,
    _de: &mut D,
  ) -> Result<Arc<dyn Source>, DeserializeError> {
    let TypeWrapper { type_name, bytes } = crate::from_bytes(field, &())?;
    match type_name.as_str() {
      // TODO change to enum
      "RawSource" => Ok(Arc::new(RawSource::from(bytes))),
      // TODO save original source name
      "OriginalSource" => Ok(Arc::new(OriginalSource::new(
        "a",
        String::from_utf8(bytes).expect("unexpected bytes"),
      ))),
      "SourceMapSource" => Ok(Arc::new(SourceMapSource::new(WithoutOriginalOptions {
        value: String::from_utf8(bytes).expect("unexpected bytes"),
        name: String::from("a"),
        source_map: SourceMap::default(),
      }))),
      _ => Err(DeserializeError::MessageError("unsupported box source")),
    }
  }
}
